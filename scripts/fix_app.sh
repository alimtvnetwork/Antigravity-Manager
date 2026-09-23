#!/bin/bash

APP_PATH="/Applications/Antigravity Tools.app"

echo "🛠️  Fixing 'Antigravity Tools is damaged' warning..."

if [ -d "$APP_PATH" ]; then
    echo "📍 Found application: $APP_PATH"
    echo "🔑 Admin permissions required to remove quarantine attribute..."

    sudo xattr -rd com.apple.quarantine "$APP_PATH"

    if [ $? -eq 0 ]; then
        echo "✅ Fix successful! The app should now launch normally."
    else
        echo "❌ Fix failed. Please check your password and file permissions."
    fi
else
    echo "⚠️  Application not found. Please confirm the app is installed in '/Applications'."
    echo "   If installed elsewhere, run: sudo xattr -rd com.apple.quarantine /path/to/app"
fi
