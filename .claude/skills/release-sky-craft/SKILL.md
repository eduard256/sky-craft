---
name: release-sky-craft
description: Release Sky Craft - build server Docker image, push to Docker Hub, create git tag, trigger GitHub Actions client builds, update release notes. Use when user says "release", "publish", "deploy new version", or "/release-sky-craft".
argument-hint: "[version]"
---

# Release Sky Craft

Full release pipeline: server Docker image + client cross-platform builds via GitHub Actions.

## Step 1: Check working tree

Run `git status` to check for uncommitted changes.
- If clean: proceed
- If dirty: ask the user what to do (commit, stash, or abort)

## Step 2: Determine version

- Run `git tag --sort=-v:refname | head -5` to find the latest version tag
- Auto-increment patch: if latest is `v0.0.1` → next is `v0.0.2`
- If user provided a version argument, use that instead
- Store version in variable (e.g. `v0.0.2`)

## Step 3: Check what changed

Quickly review what changed since last tag using `git log --oneline <last_tag>..HEAD`.
Do NOT use agents or read files for this -- just the git log. If you already know from the conversation, skip this step.

## Step 4: Build server Docker image (if server changed)

Check if server code changed since last tag:
```bash
git diff --name-only <last_tag>..HEAD -- server/ common/
```

- If changed: build and push new Docker image:
  ```bash
  docker build -f server/Dockerfile -t eduard256/skycraft-server:<version> -t eduard256/skycraft-server:latest .
  docker push eduard256/skycraft-server:<version>
  docker push eduard256/skycraft-server:latest
  ```
- If NOT changed: skip build, note the existing latest tag

## Step 5: Push to GitHub and trigger client builds

```bash
git tag <version>
git push origin main
git push origin <version>
```

This triggers the GitHub Actions workflow `build-client.yml` which builds the client for:
- Linux x86_64
- Windows x86_64
- macOS ARM64

## Step 6: Wait for GitHub Actions

Run `gh run list --limit 1` to get the run ID, then periodically check:
```bash
gh run view <run_id> --json status,conclusion
```

Wait until all jobs complete. Report any failures.

## Step 7: Update release notes

After GitHub Actions creates the release, update it to include Docker Hub link:
```bash
gh release edit <version> --notes "$(cat <<'EOF'
## Downloads

### Client
Download for your platform from the assets below.

### Server
```
docker pull eduard256/skycraft-server:<version>
docker run -d -p 35565:35565/udp eduard256/skycraft-server:<version>
```

## Changes
<git log summary from step 3>
EOF
)"
```

## Step 8: Confirm

Print summary:
- Version released
- Docker Hub: `eduard256/skycraft-server:<version>`
- GitHub Release: link to release page
- Client builds: Linux, Windows, macOS status
