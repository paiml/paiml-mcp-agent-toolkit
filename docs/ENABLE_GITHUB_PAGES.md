# How to Enable GitHub Pages

To fix the "Deploy Documentation" workflow failure, you need to enable GitHub Pages for this repository.

## Steps to Enable GitHub Pages:

1. Go to the repository settings: https://github.com/paiml/paiml-mcp-agent-toolkit/settings

2. Scroll down to the "Pages" section in the left sidebar

3. Under "Source", select:
   - **Source**: Deploy from a branch
   - **Branch**: Select "gh-pages" (we'll create this)
   - **Folder**: / (root)

4. Click "Save"

## Alternative: GitHub Actions Deployment

If you prefer to use GitHub Actions deployment (recommended):

1. In the Pages settings, under "Source", select:
   - **Source**: GitHub Actions

2. The workflow will automatically work on the next push

## Current Status:

The documentation workflow (`.github/workflows/docs.yml`) is already configured correctly to deploy documentation using GitHub Actions. It just needs Pages to be enabled in the repository settings.

Once enabled, the documentation will be available at:
https://paiml.github.io/paiml-mcp-agent-toolkit/