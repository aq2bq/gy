use crate::*;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct Store {
    pub root: PathBuf,
    pub cwd: PathBuf,
    pub config: Config,
    pub nodes: BTreeMap<String, Node>,
    counters: BTreeMap<String, u64>,
    _lock: File,
}
#[derive(Serialize, Deserialize)]
struct Journal {
    files: BTreeMap<PathBuf, String>,
}
fn atomic_write(path: &Path, data: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::corrupt("The output path has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(data.as_bytes())?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .map_err(|e| Error::corrupt(e.to_string()))?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
fn local_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}
fn checked_destination(root: &Path, path: &Path) -> Result<PathBuf> {
    if !local_path(path) {
        return Err(Error::input(format!(
            "Output must be a relative path inside the ledger: {}",
            path.display()
        )));
    }
    let mut destination = root.to_owned();
    for component in path.components() {
        destination.push(component);
        match fs::symlink_metadata(&destination) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(Error::corrupt(format!(
                    "Symbolic links inside the ledger are not supported: {}",
                    destination.display()
                )));
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(destination)
}
fn lock(root: &Path) -> Result<File> {
    let f = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".gy.lock"))?;
    f.lock_exclusive()?;
    Ok(f)
}
fn walk(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let e = entry?;
        if e.file_type()?.is_symlink() {
            return Err(Error::corrupt(format!(
                "Symbolic links inside the ledger are not supported: {}",
                e.path().display()
            )));
        }
        if e.file_type()?.is_dir() {
            walk(&e.path(), out)?;
        } else if e.path().extension().is_some_and(|s| s == "md") {
            out.push(e.path());
        }
    }
    Ok(())
}
impl Store {
    pub fn discover(cwd: &Path) -> Result<PathBuf> {
        for dir in cwd.ancestors() {
            let marker = dir.join(".gy-dir");
            if marker.is_file() {
                let value = fs::read_to_string(marker)?;
                let root = dir.join(value.trim());
                if !root.join("gy.toml").is_file() {
                    return Err(Error::corrupt(
                        "The directory referenced by .gy-dir has no gy.toml",
                    ));
                }
                return Ok(root.canonicalize()?);
            }
            if dir.join("gy.toml").is_file() {
                return Ok(dir.canonicalize()?);
            }
            if dir.join("docs/ledger/gy.toml").is_file() {
                return Ok(dir.join("docs/ledger").canonicalize()?);
            }
        }
        Err(Error::input(
            "Ledger not found. Run gy init <scope> inside the project",
        ))
    }
    pub fn init(cwd: &Path, scope: &str, parent_issue: Option<u64>) -> Result<Self> {
        if !safe_component(scope) {
            return Err(Error::input(
                "Use a scope identifier that is valid as a directory name",
            ));
        }
        let existing = Self::discover(cwd);
        let root = match &existing {
            Ok(root) => root.clone(),
            Err(e) if e.code == 2 => cwd.join("docs/ledger"),
            Err(e) => return Err(e.clone()),
        };
        fs::create_dir_all(&root)?;
        {
            let _lock = lock(&root)?;
            if !root.join("gy.toml").exists() {
                atomic_write(
                    &root.join("gy.toml"),
                    "[render]\nsplit_threshold = 100\noutput = \"{scope}/README.md\"\n\n[lint]\n",
                )?;
            }
            for (_, _, dir) in KINDS {
                fs::create_dir_all(root.join(scope).join(dir))?;
            }
        }
        if existing.is_err() {
            atomic_write(&cwd.join(".gy-dir"), "docs/ledger\n")?;
            let agents = cwd.join("AGENTS.md");
            let line = "gy ledger: docs/ledger/ (read `gy cheatsheet` and `gy handover` at the start of each session).";
            let mut original = if agents.exists() {
                fs::read_to_string(&agents)?
            } else {
                String::new()
            };
            if !original.contains(line) {
                if !original.is_empty() && !original.ends_with('\n') {
                    original.push('\n');
                }
                original.push_str(line);
                original.push('\n');
                atomic_write(&agents, &original)?;
            }
        }
        let mut store = Self::open(cwd)?;
        let entry = store.config.scopes.entry(scope.into()).or_default();
        if let Some(p) = parent_issue {
            entry.parent_issue = Some(p);
        }
        store.commit()?;
        Ok(store)
    }
    pub fn open(cwd: &Path) -> Result<Self> {
        let cwd = cwd.canonicalize()?;
        let root = Self::discover(&cwd)?;
        let lock = lock(&root)?;
        let pending = root.join(".gy-transaction.json");
        if pending.exists() {
            let j: Journal = serde_json::from_str(&fs::read_to_string(&pending)?)
                .map_err(|e| Error::corrupt(format!("Invalid pending update journal: {e}")))?;
            for (path, data) in j.files {
                if !local_path(&path) {
                    return Err(Error::corrupt(
                        "The update journal contains a path outside the ledger",
                    ));
                }
                atomic_write(&checked_destination(&root, &path)?, &data)?;
            }
            fs::remove_file(pending)?;
        }
        let config: Config = toml::from_str(&fs::read_to_string(root.join("gy.toml"))?)
            .map_err(|e| Error::corrupt(format!("gy.toml: {e}")))?;
        for (rule, setting) in &config.lint {
            if !(1..=13).any(|n| rule == &format!("L{n}")) && rule != "edges" {
                return Err(Error::corrupt(format!("Unknown lint rule: {rule}")));
            }
            setting.severity()?;
        }
        if config
            .import
            .scope_note_section
            .as_ref()
            .is_some_and(|s| s.trim().is_empty())
        {
            return Err(Error::corrupt(
                "import.scope_note_section must be a nonempty heading title",
            ));
        }
        if config.render.split_threshold == 0 {
            return Err(Error::corrupt("render.split_threshold must be at least 1"));
        }
        let mut nodes = BTreeMap::new();
        for scope in fs::read_dir(&root)? {
            let scope = scope?;
            if scope.file_type()?.is_symlink() {
                return Err(Error::corrupt(format!(
                    "Symbolic links inside the ledger are not supported: {}",
                    scope.path().display()
                )));
            }
            if !scope.file_type()?.is_dir() {
                continue;
            }
            for (kind, _, dir) in KINDS {
                let mut paths = vec![];
                walk(&scope.path().join(dir), &mut paths)?;
                for path in paths {
                    let node = Node::parse(&fs::read_to_string(&path)?, path.clone())?;
                    if node.kind() != *kind || scope.file_name().to_str() != Some(node.scope()) {
                        return Err(Error::corrupt(format!(
                            "{}: type / scope does not match the file location",
                            path.display()
                        )));
                    }
                    let id = node.id().to_owned();
                    if nodes.insert(id.clone(), node).is_some() {
                        return Err(Error::corrupt(format!(
                            "Duplicate ID {id}. IDs must be unique across all scopes"
                        )));
                    }
                }
            }
        }
        let p = root.join(".gy-ids.json");
        let mut counters: BTreeMap<String, u64> = if p.exists() {
            serde_json::from_str(&fs::read_to_string(p)?)
                .map_err(|e| Error::corrupt(e.to_string()))?
        } else {
            BTreeMap::new()
        };
        for node in nodes.values() {
            let (_, prefix, _) = KINDS.iter().find(|k| k.0 == node.kind()).unwrap();
            let n = node.id().trim_start_matches(prefix).parse::<u64>().unwrap();
            let counter = counters.entry(node.kind().into()).or_default();
            *counter = (*counter).max(n);
        }
        Ok(Self {
            root,
            cwd,
            config,
            nodes,
            counters,
            _lock: lock,
        })
    }
    pub fn scope(&self, explicit: Option<&str>) -> Result<String> {
        if let Some(scope) = explicit {
            if safe_component(scope) && self.root.join(scope).is_dir() {
                return Ok(scope.into());
            }
            return Err(Error::input(format!(
                "Scope {scope} does not exist. Create it with gy init {scope}"
            )));
        }
        if let Ok(rel) = self.cwd.strip_prefix(&self.root) {
            if let Some(s) = rel.components().next().and_then(|c| c.as_os_str().to_str()) {
                if self.root.join(s).is_dir() {
                    return Ok(s.into());
                }
            }
        }
        Err(Error::input(
            "Cannot resolve the write scope from the current directory. Specify --scope <name>",
        ))
    }
    pub fn node(&self, id: &str) -> Result<&Node> {
        self.nodes.get(id).ok_or_else(|| {
            Error::input(format!(
                "Referenced node {id} does not exist. Use gy find to check existing IDs"
            ))
        })
    }
    pub fn typed(&self, id: &str, kind: &str) -> Result<&Node> {
        let n = self.node(id)?;
        if !kind_matches(n.kind(), kind) {
            Err(Error::input(format!("{id} is not a {kind}")))
        } else {
            Ok(n)
        }
    }
    pub fn create(
        &mut self,
        kind: &str,
        title: &str,
        scope: &str,
        issue: Option<u64>,
    ) -> Result<String> {
        self.scope(Some(scope))?;
        if title.trim().is_empty() {
            return Err(Error::input("A descriptive title is required"));
        }
        let (_, prefix, directory) = KINDS
            .iter()
            .find(|k| k.0 == kind)
            .ok_or_else(|| Error::input("Unknown node type"))?;
        let n = if kind == "requirement" {
            issue.filter(|n| *n > 0).ok_or_else(|| Error::input("--issue <number> is required to identify a requirement. gy does not query GitHub"))?
        } else {
            self.counters
                .get(kind)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| Error::corrupt("ID limit reached"))?
        };
        let id = format!("{prefix}{n}");
        if self.nodes.contains_key(&id) {
            return Err(Error::input(format!("{id} already exists")));
        }
        self.counters.insert(
            kind.into(),
            self.counters.get(kind).copied().unwrap_or(0).max(n),
        );
        let mut node = Node::new(&id, kind, title, scope);
        node.path = self
            .root
            .join(scope)
            .join(directory)
            .join(format!("{}.md", id.trim_start_matches('#')));
        self.nodes.insert(id.clone(), node);
        Ok(id)
    }
    pub fn commit(&self) -> Result<()> {
        let mut files = BTreeMap::new();
        for node in self.nodes.values() {
            let data = node.markdown()?;
            Node::parse(&data, node.path.clone())?;
            if fs::read_to_string(&node.path).ok().as_deref() != Some(&data) {
                files.insert(
                    node.path
                        .strip_prefix(&self.root)
                        .map_err(|e| Error::corrupt(e.to_string()))?
                        .to_owned(),
                    data,
                );
            }
        }
        files.insert(
            PathBuf::from(".gy-ids.json"),
            serde_json::to_string_pretty(&self.counters).unwrap(),
        );
        files.insert(
            PathBuf::from("gy.toml"),
            toml::to_string_pretty(&self.config).map_err(|e| Error::corrupt(e.to_string()))?,
        );
        self.write_files(files)
    }
    pub fn write_files(&self, files: BTreeMap<PathBuf, String>) -> Result<()> {
        for path in files.keys() {
            checked_destination(&self.root, path)?;
        }
        let journal = Journal { files };
        let pending = self.root.join(".gy-transaction.json");
        atomic_write(&pending, &serde_json::to_string(&journal).unwrap())?;
        for (path, data) in journal.files {
            atomic_write(&checked_destination(&self.root, &path)?, &data)?;
        }
        fs::remove_file(pending)?;
        Ok(())
    }
}
