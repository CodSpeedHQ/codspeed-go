# Release Process

This document describes the steps to release a new version of codspeed-go.

## Steps

1. **Set the new version number**

   ```bash
   export NEW_CODSPEED_GO_VERSION=0.4.2
   ```

   Replace `0.4.2` with the desired version number.

2. **Generate the changelog**

   ```bash
   git cliff --tag "v$NEW_CODSPEED_GO_VERSION" -o CHANGELOG.md
   ```

   If you haven't installed `git-cliff`, install it with `cargo binstall git-cliff`.

   This will update the CHANGELOG.md file with all changes since the last release.

3. **Update the go-runner version**

   Edit [go-runner/Cargo.toml](go-runner/Cargo.toml) and change the `version` field to match `NEW_CODSPEED_GO_VERSION`:

   ```toml
   [package]
   version = "0.4.2"  # Update this line
   ```

4. **Commit the changes**

   ```bash
   git add .
   git commit -m "chore: release v$NEW_CODSPEED_GO_VERSION"
   ```

5. **Create an annotated git tag**

   ```bash
   git tag -a "v$NEW_CODSPEED_GO_VERSION" -m "Release v$NEW_CODSPEED_GO_VERSION"
   ```

   Note: Use an annotated tag (with `-a` and `-m` flags) so that `--follow-tags` will push it automatically.

6. **Push with tags**
   ```bash
   git push --follow-tags
   ```
   This will push the commit and the annotated tag to the remote repository in one command.
