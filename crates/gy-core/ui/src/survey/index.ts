import { initBlockers } from './blockers';
import { initOverview } from './overview';
import { initProgress } from './progress';
export function initSurvey() { initOverview(); initBlockers(); initProgress(); }
