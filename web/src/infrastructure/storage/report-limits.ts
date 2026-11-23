export const SCRIPT_FILE_LIMIT = 50_000_000;
export const REPORT_FILE_LIMIT = 100_000_000;
export const REPORT_JSON_LIMIT = 100_000_000;

export function checkReportFileSize(size: number, limit = REPORT_FILE_LIMIT) {
  if (!Number.isSafeInteger(size) || size < 0 || size > limit)
    throw new Error('Report file exceeds the 100 MB file limit.');
}

export function checkReportJsonSize(size: number, limit = REPORT_JSON_LIMIT) {
  if (!Number.isSafeInteger(size) || size <= 0 || size > limit)
    throw new Error('Report JSON exceeds the decoded size limit (100 MB) or is empty.');
}
