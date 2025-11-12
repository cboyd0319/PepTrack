# Google OAuth Setup Guide for PepTrack

## Overview

This guide walks you through setting up Google OAuth 2.0 credentials to enable Google Drive backup functionality in PepTrack. This is a **one-time setup** that you (the developer) will perform. Once configured, all users of your PepTrack installation will be able to backup to Google Drive without needing their own Google Cloud accounts.

**Time Required**: 15-20 minutes
**Technical Level**: Intermediate
**Prerequisites**: Google account with access to Google Cloud Console

---

## Table of Contents

1. [Understanding the OAuth Flow](#understanding-the-oauth-flow)
2. [Step-by-Step Setup](#step-by-step-setup)
3. [Configuring Your Application](#configuring-your-application)
4. [Security Best Practices](#security-best-practices)
5. [Troubleshooting](#troubleshooting)
6. [FAQ](#faq)

---

## Understanding the OAuth Flow

### What is OAuth 2.0?

OAuth 2.0 is an authorization framework that allows PepTrack to access Google Drive on behalf of users **without** requiring their Google passwords. Users grant permission through Google's secure interface.

### Developer-Managed OAuth (What We're Using)

In this setup:
- **You (developer)** create ONE set of OAuth credentials in Google Cloud
- **You** hard-code these credentials into the application
- **Users** authenticate through Google's interface, granting PepTrack permission to access their Drive
- **Users** can revoke access at any time through their Google Account settings

### What Data Does PepTrack Access?

PepTrack requests the **minimum necessary scope**:
- `https://www.googleapis.com/auth/drive.file` - Only access to files that PepTrack creates
- PepTrack **cannot** see, read, or modify any other files in the user's Google Drive

---

## Step-by-Step Setup

### Step 1: Create a Google Cloud Project

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Sign in with your Google account
3. Click the project dropdown at the top (says "Select a project")
4. Click **"NEW PROJECT"** in the top-right
5. Fill out the form:
   - **Project name**: `PepTrack` (or any name you prefer)
   - **Organization**: Leave as "No organization" (unless you have one)
6. Click **"CREATE"**
7. Wait for the project to be created (takes ~30 seconds)
8. Select your new project from the dropdown

### Step 2: Enable Google Drive API

1. In the left sidebar, navigate to **"APIs & Services" > "Library"**
   - Or use the search bar and type "API Library"
2. Search for **"Google Drive API"**
3. Click on **"Google Drive API"** from the results
4. Click the blue **"ENABLE"** button
5. Wait for the API to be enabled (takes ~10 seconds)

### Step 3: Configure OAuth Consent Screen

This is what users see when they authorize PepTrack to access their Drive.

1. Navigate to **"APIs & Services" > "OAuth consent screen"**
2. Choose **"External"** (unless you have a Google Workspace organization)
3. Click **"CREATE"**

#### OAuth Consent Screen - Page 1 (App Information)

Fill out the required fields:

- **App name**: `PepTrack`
- **User support email**: Your email address (dropdown)
- **App logo** (optional): Upload a logo if you have one (120x120px minimum)
- **Application home page** (optional): Leave blank or add your website
- **Application privacy policy link** (optional): Leave blank for now
- **Application terms of service link** (optional): Leave blank for now
- **Authorized domains** (optional): Leave blank
- **Developer contact information**: Your email address

Click **"SAVE AND CONTINUE"**

#### OAuth Consent Screen - Page 2 (Scopes)

1. Click **"ADD OR REMOVE SCOPES"**
2. In the filter box, search for: `drive.file`
3. Check the box next to:
   - `https://www.googleapis.com/auth/drive.file` - "See, edit, create, and delete only the specific Google Drive files you use with this app"
4. Click **"UPDATE"** at the bottom
5. Verify the scope appears in the "Your sensitive scopes" table
6. Click **"SAVE AND CONTINUE"**

#### OAuth Consent Screen - Page 3 (Test Users)

For development/testing:
1. Click **"ADD USERS"**
2. Add your email address (and any other test users)
3. Click **"ADD"**
4. Click **"SAVE AND CONTINUE"**

> **Note**: While in "Testing" mode, only added test users can authorize the app. To allow ALL users, you'll need to publish the app (see [Publishing Your App](#publishing-your-app) below).

#### OAuth Consent Screen - Page 4 (Summary)

1. Review your settings
2. Click **"BACK TO DASHBOARD"**

### Step 4: Create OAuth Credentials

1. Navigate to **"APIs & Services" > "Credentials"**
2. Click **"CREATE CREDENTIALS"** at the top
3. Select **"OAuth client ID"**
4. Configure the OAuth client:
   - **Application type**: Select **"Desktop app"**
   - **Name**: `PepTrack Desktop Client` (or any name)
5. Click **"CREATE"**

### Step 5: Retrieve Your Credentials

A dialog will appear showing your credentials:

```
Client ID: 123456789012-abcdefghijklmnopqrstuvwxyz123456.apps.googleusercontent.com
Client Secret: GOCSPX-abcdefghijklmnopqrstuvwx
```

**IMPORTANT**:
- Click **"DOWNLOAD JSON"** to save a backup copy
- Click **"OK"** to close the dialog
- **Keep these credentials secure** - see [Security Best Practices](#security-best-practices)

---

## Configuring Your Application

### Step 6: Add Credentials to PepTrack

1. Open the file: `frontend/src/components/GoogleDriveBackup.vue`
2. Locate the configuration section near the top (around line 15-20):

```typescript
// 🔐 OAUTH CREDENTIALS - DEVELOPER CONFIGURATION
const GOOGLE_OAUTH_CONFIG: DriveOAuthConfig = {
  clientId: "YOUR_CLIENT_ID_HERE.apps.googleusercontent.com",
  clientSecret: "YOUR_CLIENT_SECRET_HERE",
};
```

3. Replace the placeholder values with your actual credentials:

```typescript
const GOOGLE_OAUTH_CONFIG: DriveOAuthConfig = {
  clientId: "123456789012-abcdefghijklmnopqrstuvwxyz123456.apps.googleusercontent.com",
  clientSecret: "GOCSPX-abcdefghijklmnopqrstuvwx",
};
```

4. Save the file

### Step 7: Build and Test

1. Rebuild the application:
```bash
cd frontend
npm run build
cd ..
cargo tauri build
```

2. Launch the application and test:
   - Navigate to **Settings > Backup & Restore > Cloud Sync**
   - Click **"Connect to Google Drive"**
   - A browser window should open with Google's authorization page
   - Authorize the app
   - The connection status should update to "Connected"

---

## Security Best Practices

### ✅ DO

1. **Keep Client Secret Private**
   - Never commit it to public repositories
   - Add `GoogleDriveBackup.vue` to `.gitignore` if needed
   - Consider using environment variables for production

2. **Use Minimal Scopes**
   - The `drive.file` scope only accesses files PepTrack creates
   - Never request `drive` (full access) or `drive.readonly` unless absolutely necessary

3. **Monitor Usage**
   - Regularly check Google Cloud Console for unusual API activity
   - Set up usage alerts in the Cloud Console

4. **Revoke Compromised Credentials**
   - If credentials are exposed, immediately delete them in Cloud Console
   - Create new credentials and update your application

5. **Inform Users**
   - Clearly explain what data PepTrack accesses
   - Provide instructions for users to revoke access if desired

### ❌ DON'T

1. **Don't Share Client Secret Publicly**
   - Never post in forums, GitHub issues, or support tickets
   - Don't include in screenshots or documentation

2. **Don't Request Excessive Scopes**
   - Only use the scopes your application actually needs

3. **Don't Ignore Security Updates**
   - Keep dependencies updated (especially OAuth libraries)
   - Monitor Google's OAuth security advisories

### Handling Credentials in Version Control

If you're using Git and plan to share your code:

**Option 1: Environment Variables (Recommended for Production)**

1. Create a `.env` file (add to `.gitignore`):
```
VITE_GOOGLE_CLIENT_ID=your_client_id_here
VITE_GOOGLE_CLIENT_SECRET=your_client_secret_here
```

2. Update `GoogleDriveBackup.vue`:
```typescript
const GOOGLE_OAUTH_CONFIG: DriveOAuthConfig = {
  clientId: import.meta.env.VITE_GOOGLE_CLIENT_ID || "YOUR_CLIENT_ID_HERE",
  clientSecret: import.meta.env.VITE_GOOGLE_CLIENT_SECRET || "YOUR_CLIENT_SECRET_HERE",
};
```

**Option 2: Separate Config File**

1. Create `frontend/src/config/google-oauth.config.ts` (add to `.gitignore`):
```typescript
export const GOOGLE_OAUTH_CONFIG = {
  clientId: "your_client_id_here",
  clientSecret: "your_client_secret_here",
};
```

2. Import in `GoogleDriveBackup.vue`:
```typescript
import { GOOGLE_OAUTH_CONFIG } from '@/config/google-oauth.config';
```

3. Create a template file to commit: `google-oauth.config.template.ts`

---

## Publishing Your App

### Testing vs. Production

Your OAuth app starts in **"Testing"** mode, which means:
- Only users you explicitly add as "Test Users" can authorize
- Tokens expire after 7 days (users need to re-authorize)

To allow **all users** to connect:

### Steps to Publish

1. Go to **"APIs & Services" > "OAuth consent screen"**
2. Click **"PUBLISH APP"** button
3. Review the warning and click **"CONFIRM"**

**Important**:
- Google may require verification if you request sensitive scopes
- The `drive.file` scope is considered "restricted" and may trigger verification
- Verification can take several weeks
- For personal use or small user bases, staying in "Testing" mode is acceptable

### Verification Requirements

If Google requires verification, you'll need:
- A privacy policy hosted on your domain
- Justification for the scopes you're requesting
- A demo video showing how your app uses the scopes
- YouTube video walkthrough of your app

**Alternative**: Keep the app in "Testing" mode and manually add users (up to 100 test users allowed).

---

## Troubleshooting

### Error: "Access blocked: This app's request is invalid"

**Cause**: Redirect URI mismatch or OAuth consent screen not configured

**Solution**:
1. Verify OAuth consent screen is fully configured
2. Check that application type is "Desktop app"
3. Ensure you completed all 4 pages of consent screen setup

### Error: "Invalid client ID"

**Cause**: Client ID is incorrect or not properly copied

**Solution**:
1. Go to Cloud Console > Credentials
2. Click on your OAuth 2.0 Client ID
3. Copy the Client ID again (it should end with `.apps.googleusercontent.com`)
4. Paste it exactly into `GoogleDriveBackup.vue`

### Error: "Unauthorized client"

**Cause**: Client Secret is incorrect

**Solution**:
1. Verify the Client Secret in Cloud Console
2. If lost, you can reset it (this will invalidate the old one)
3. Update `GoogleDriveBackup.vue` with the new secret

### Error: "Access denied: drive.file scope not granted"

**Cause**: User didn't grant permission or scope not configured

**Solution**:
1. Verify scope `drive.file` is added in OAuth consent screen
2. User needs to authorize again and click "Allow" for all permissions

### Users See "This app isn't verified"

**Cause**: App is in testing mode or not verified by Google

**Solution**:
- Click "Advanced" → "Go to PepTrack (unsafe)" to continue
- Or publish the app and complete verification (see [Publishing Your App](#publishing-your-app))

### Tokens Expire After 7 Days

**Cause**: App is in "Testing" mode

**Solution**:
- Publish the app (tokens will last longer)
- Or accept that test users need to re-authorize weekly

---

## FAQ

### Q: Can users see my Client Secret?

**A**: No. The Client Secret is embedded in your compiled application. While determined users could extract it from the binary, this is expected behavior for desktop applications. The secret is used to prove your app's identity to Google, but users still need to authorize access to their own Google Drive.

### Q: What happens if my credentials are compromised?

**A**:
1. Immediately delete the credentials in Google Cloud Console
2. Create new credentials
3. Update your application with the new credentials
4. Rebuild and redistribute your application

Users will need to re-authorize with the new credentials.

### Q: How much does this cost?

**A**: Google Drive API is **free** for most use cases:
- Free tier: 1 billion queries per day
- PepTrack's backup feature uses minimal API calls
- You'll likely never exceed the free tier

### Q: Can users revoke PepTrack's access?

**A**: Yes! Users can revoke access at any time:
1. Go to [Google Account - Third-party apps & services](https://myaccount.google.com/permissions)
2. Find "PepTrack" in the list
3. Click "Remove Access"

### Q: Do I need a different OAuth app for each user?

**A**: No! That's the point of this developer-managed approach. You create ONE set of credentials, and all users authorize through that single OAuth app.

### Q: What if I want to distribute PepTrack publicly?

**A**:
- You should publish the OAuth app and complete Google's verification process
- Consider adding proper Terms of Service and Privacy Policy
- Make sure to comply with Google's API Terms of Service
- Consider liability and data protection requirements

### Q: Can I use the same credentials for multiple installations?

**A**: Yes, the same credentials work for all instances of PepTrack. However:
- Each user still authorizes independently
- Each user's Drive data is kept separate
- Revoking credentials affects all installations

---

## Additional Resources

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Google Drive API Quickstart](https://developers.google.com/drive/api/quickstart/nodejs)
- [OAuth 2.0 for Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app)
- [Google API Terms of Service](https://developers.google.com/terms)
- [Drive API Usage Limits](https://developers.google.com/drive/api/guides/limits)

---

## Support

If you encounter issues not covered in this guide:

1. Check the [Troubleshooting](#troubleshooting) section above
2. Review Google's OAuth 2.0 documentation
3. Check the PepTrack GitHub issues for similar problems
4. Open a new GitHub issue with:
   - Detailed error messages
   - Steps to reproduce
   - **DO NOT include your Client ID or Secret**

---

## Security Disclosure

If you discover a security vulnerability in PepTrack's OAuth implementation, please report it responsibly:

- **DO NOT** open a public GitHub issue
- Email the maintainer directly with details
- Allow time for a fix before public disclosure

---

*Last Updated: 2025-11-12*
