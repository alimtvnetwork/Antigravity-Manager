cask "antigravity-tools" do
  version "4.205.0"
  sha256 :no_check

  name "Antigravity Manager Tools By Alim"
  desc "Enterprise-Grade AI Account Management and Protocol Proxy Gateway"
  homepage "https://github.com/alimtvnetwork/Antigravity-Manager"

  on_macos do
    arch intel: "x64", arm: "aarch64"

    url "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v#{version}/Antigravity.Manager.Tools_#{version}_#{arch}.dmg"

    app "Antigravity Manager Tools.app"

    postflight do
      system_command "/usr/bin/xattr",
                     args:         ["-rd", "com.apple.quarantine", "#{appdir}/Antigravity Manager Tools.app"],
                     must_succeed: false
    end

    zap trash: [
      "~/Library/Application Support/com.alimtvnetwork.antigravity-tools",
      "~/Library/Caches/com.alimtvnetwork.antigravity-tools",
      "~/Library/Preferences/com.alimtvnetwork.antigravity-tools.plist",
      "~/Library/Saved Application State/com.alimtvnetwork.antigravity-tools.savedState",
    ]
  end

  on_linux do
    arch arm: "arm64", intel: "amd64"

    url "https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v#{version}/Antigravity.Manager.Tools_#{version}_#{arch}.deb"
  end
end
