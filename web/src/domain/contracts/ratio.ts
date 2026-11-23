/** Canonical `"<numerator>/<denominator>"` decimal string for Rust `Ratio`. */
export type Ratio = string;

/** Canonical `"<numerator>/<denominator>"` decimal string for Rust `ExactFraction` / `WideFraction`. */
export type ExactFraction = string;

/** Object shape retained ONLY for external `Scalar` and research delta pairs (`SignedFraction`). */
export interface SignedFraction {
  numerator: string;
  denominator: string;
}
