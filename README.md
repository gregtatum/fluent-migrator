# Fluent Migrator

Migrate Firefox [.dtd](https://searchfox.org/mozilla-central/search?q=&path=*.dtd) and [.properties](https://searchfox.org/mozilla-central/search?q=&path=*.properties) files to [Fluent](https://projectfluent.org/). See [arewefluentyet.com](https://www.arewefluentyet.com/) for the remaining files that need to be migrated.

## Usage

```sh
# Install or update
cargo install --git https://github.com/gregtatum/fluent-migrator

# See the help
fluent-migrator --help

# Output a migration to std out
fluent-migrator path/to/file.dtd
fluent-migrator path/to/file.properties

# Save out to path/to/file.ftl
fluent-migrator --save path/to/file.dtd

# Overwrite a previous migration
fluent-migrator --save --overwrite path/to/file.dtd

# Migrate multiple files at once
fluent-migrator --save file1.dtd file2.properties file3.dtd
```
## Recipes

For migration recipes see [recipe-dtd.sh](recipe-dtd.sh) and [recipe-properties.sh](recipe-properties.sh).
