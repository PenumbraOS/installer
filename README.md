# PenumbraOS Installer

Installer for PenumbraOS on Humane Ai Pin.

> [!CAUTION]
> This installer is not complete and PenumbraOS is not ready for end users. DO NOT ATTEMPT TO INSTALL unless you're ready to troubleshoot and mess up your Pin.

## Usage

### Basic Commands

```bash
# Install all PenumbraOS components
installer install

# Uninstall all PenumbraOS components
installer uninstall

# Install only specific repositories
installer install --repos pinitd,mabl

# Install from local download cache
installer install --cache_dir cache

# Install using a GitHub PAT for downloads
installer install --github-token [SOME_PAT]

# Download to local cache
installer download --cache_dir cache
```
