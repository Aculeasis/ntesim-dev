export interface NtesimLosslessJsonApi {
  parseLosslessJson(text: string): unknown;
}

declare global {
  var NtesimLosslessJson: NtesimLosslessJsonApi | undefined;
}
