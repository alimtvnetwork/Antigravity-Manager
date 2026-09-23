cask "antigravity-tools" do
  version "4.65.2"
  sha256 :no_check

  name "Antigravity Manager Tools By Alim"
  desc "Enterprise-Grade AI Account Management and Protocol Proxy Gateway"
  homepage "https://github.com/alimtvnetwork/Antigravity-Manager"

  on_macos do
    arch intel: "x64", arm: "aarch64"

    url "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v#{version}/agm-alim_#{version}_#{arch}.dmg"

    app "agm-alim.app"

    postflight do
      system_command "/usr/bin/xattr",
                     args:         ["-rd", "com.apple.quarantine", "#{appdir}/agm-alim.app"],
                     must_succeed: false
    end

    zap trash: [
      "~/Library/Application Support/com.lbjlaq.antigravity-tools",
      "~/Library/Caches/com.lbjlaq.antigravity-tools",
      "~/Library/Preferences/com.lbjlaq.antigravity-tools.plist",
      "~/Library/Saved Application State/com.lbjlaq.antigravity-tools.savedState",
    ]
  end

  on_linux do
    arch arm: "aarch64", intel: "amd64"

    url "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v#{version}/agm-alim_#{version}_#{arch}.AppImage"
    binary "agm-alim_#{version}_#{arch}.AppImage", target: "agm-alim"

    preflight do
      set_permissions "#{staged_path}/agm-alim_#{version}_#{arch}.AppImage", "0755"
    end
  end
end
