/* The one date component (n-648d, d-b1f8): every screen shows a stored instant
   in the reader's own place. A value is an RFC 3339 string or epoch seconds.
   A part: it reaches for neither the document nor an event listener, and it
   judges nothing else. */
(function () {
  const LOCALE = lang => (lang === 'ja' ? 'ja-JP' : 'en-GB');
  const pad = value => String(value).padStart(2, '0');

  /* The instant a value means, or null when it is not one. A number, or an
     all-digit string, is epoch seconds; anything else is parsed as a date. */
  function instant(value) {
    if (value === null || value === undefined || value === '') return null;
    const date = typeof value === 'number' || /^-?\d+$/.test(String(value))
      ? new Date(Number(value) * 1000)
      : new Date(value);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  /* The reader's own calendar day, in the history heading's form. */
  function date(value) {
    const at = instant(value);
    if (!at) return '';
    return `${at.getFullYear()}/${pad(at.getMonth() + 1)}/${pad(at.getDate())}`;
  }

  /* The reader's own date and time, in the node page's requirement steps. */
  function dateTime(value, lang) {
    const at = instant(value);
    if (!at) return '';
    return at.toLocaleString(LOCALE(lang), {
      month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
    });
  }

  /* The reader's own time of day, in the history rows. */
  function time(value, lang) {
    const at = instant(value);
    if (!at) return '';
    return at.toLocaleTimeString(LOCALE(lang), { hour: '2-digit', minute: '2-digit' });
  }

  /* Whole days from the value's own day to today, in the reader's place; today
     or a future day reads as `today` (n-fa11). The word comes from the caller. */
  function age(value, t) {
    const at = instant(value);
    if (!at) return '';
    const now = new Date();
    const from = Date.UTC(at.getFullYear(), at.getMonth(), at.getDate());
    const to = Date.UTC(now.getFullYear(), now.getMonth(), now.getDate());
    const days = Math.round((to - from) / 86400000);
    return days > 0 ? t('unwaitedDays').replace(/\{n\}/g, days) : t('today');
  }

  window.GyTime = { date, dateTime, time, age };
})();