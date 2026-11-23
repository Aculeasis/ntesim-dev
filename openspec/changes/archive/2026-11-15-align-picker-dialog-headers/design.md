# Design

## Context

The shared dialog search selector adds a bottom margin that offsets Team list's grid alignment. SearchDialog owns every Name Search category and currently uses the default two-control heading plus a separate search field.

## Decisions

- Remove the generic search bottom margin; place spacing on the new search header instead. Searches are only used in these two dialog owners.
- Give SearchDialog a custom flex header with flexible, shrinkable search and the same transparent multiplication-sign close control as Team list. Share the icon style through a dialog-close class.
- Retain an accessible dialog name with aria-label when no visible heading exists. Existing titled dialogs continue using aria-labelledby. Avoid a hidden duplicate heading by explicitly supporting an accessible-only title in Dialog.
- Keep reference gallery cards and all search/insertion behavior intact. This request concerns the dialog header row. No engine or transport changes are needed.

## Risks / Trade-offs

- Removing the visible title could remove the accessible name → explicitly name SearchDialog and verify it in the browser.
- Shrinking search could overflow on mobile → use min-width: 0 and check 390px layout.
- CSS alignment cannot be established by static rendering → use browser geometry and screenshots in the existing browser acceptance boundary.
