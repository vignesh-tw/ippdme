## RENAMED Requirements
- FROM: `### Requirement: Helpers read term parameters`
- TO: `### Requirement: Helpers read parameters and leading tags`

## MODIFIED Requirements
### Requirement: Helpers read parameters and leading tags
A term SHALL provide lookups for the first numeric parameter and first identifier parameter by name. The library SHALL provide `leading_tag(line)` returning a tag only when the line starts with five ASCII digits followed by whitespace or end of line, so a raw line can be correlated with its reply without full parsing. It SHALL also provide `parse_term_str` for parsing a bare term without a tag or marker.

#### Scenario: Leading tag
- **WHEN** `leading_tag` is given `00007 Home()` or `00007`
- **THEN** it returns tag 7

#### Scenario: Not a tag
- **WHEN** `leading_tag` is given `0007 Home()`, `000071 Home()`, `Home()` or an empty string
- **THEN** it returns nothing

#### Scenario: Numeric parameter
- **WHEN** `GoTo(X(10.0), Y(20.0))` is queried for `X`
- **THEN** 10.0 is returned

