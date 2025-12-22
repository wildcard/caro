# GitHub Pages Configuration

## Important: Manual Setup Required

The website deployment workflow is configured and ready, but GitHub Pages needs to be enabled in the repository settings.

### Steps to Enable GitHub Pages

1. Go to the repository on GitHub: https://github.com/wildcard/caro
2. Click on **Settings** (repository settings, not account settings)
3. Navigate to **Pages** in the left sidebar
4. Under "Build and deployment":
   - **Source**: Select **GitHub Actions** from the dropdown
   - This allows the deploy-website.yml workflow to deploy the site
5. Under "Custom domain" (if using caro.sh):
   - Enter: `caro.sh`
   - Click **Save**
   - This will use the CNAME file from the website/public directory

### DNS Configuration for Custom Domain

If using the custom domain `caro.sh`, ensure DNS records point to GitHub Pages:

```
Type: A
Name: @
Value: 185.199.108.153
       185.199.109.153
       185.199.110.153
       185.199.111.153

Type: CNAME
Name: www
Value: wildcard.github.io
```

### Verification

After enabling GitHub Pages:
1. The next push to `main` that modifies `website/**` files will trigger deployment
2. You can manually trigger deployment using the "Run workflow" button in the Actions tab
3. The site should be accessible at:
   - With custom domain: https://caro.sh
   - Without custom domain: https://wildcard.github.io/caro

### Troubleshooting

If deployment fails after enabling:
- Check that the "pages: write" permission is granted to workflows
- Verify the GitHub Actions source is selected (not "Deploy from a branch")
- Check the Actions tab for detailed error messages
- Ensure the gh-pages environment exists or will be created automatically

## Current Status

- ✅ Workflow file configured (`.github/workflows/deploy-website.yml`)
- ✅ Website builds successfully
- ✅ CNAME file added for custom domain
- ⏳ GitHub Pages needs to be enabled in repository settings (manual step)
