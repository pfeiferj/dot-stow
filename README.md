# dot-stow
A dotfile manager taking some inspiration from gnu stow. Using a .stow.yml file
it creates symlinks to files in multiple root directory structures to their
indicated target locations. It also allows running scripts before and after each
source->target mapping's symlink creation.

Currently in a beta state, breaking changes are expected.

# Usage
Running `dot-stow --init` creates a script for downloading the latest binary
release of dot-stow. The script detects if running on mac or linux and downloads
the correct binary. These scripts can be committed to the git repository to
allow easily getting dot-stow onto a new system by running `sh install`. It also
creates an example .stow.yml file in the current directory. If
`dot-stow --init` is ran again it will overwrite the install scripts but not the
.stow.yml file, this allows an easy way to ensure you have the latest install
scripts in your repository.

# Suggested Directory Stucture

