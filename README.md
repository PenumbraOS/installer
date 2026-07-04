> [!CAUTION]
> This exploit path is deprecated and is no longer in use. See the [center repo](https://github.com/PenumbraOS/center) and https://penumbraos.com for current PenumbraOS work.

# PenumbraOS Installer

(Deprecated) installer for PenumbraOS on Humane Ai Pin.

## Usage

### Emergency Reboot/Bootloop Fix

```bash
installer restart
installer restart --remote-auth-url [SOME_SIGNING_URL]
```

### Basic Commands

```bash
# Install all PenumbraOS components
installer install --llm-api-url [URL] --llm-api-key [API KEY] --llm-api-model-name [NAME]

# Uninstall all PenumbraOS components
installer uninstall

# Install only specific repositories
installer install --repos pinitd,mabl

# Install from local download cache
installer install --cache-dir cache

# Install using a GitHub PAT for downloads
installer install --github-token [SOME_PAT]

# Install using a remote signing server
installer install --remote-auth-url [SOME_SIGNING_URL]

# Download to local cache
installer download --cache-dir cache

# Dump current logs to file
installer dump-logs

# Dump logs, streaming to file until CTRL-C is pressed
installer dump-logs --stream
```
