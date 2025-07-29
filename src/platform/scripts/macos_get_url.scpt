  #!/usr/bin/osascript

  on getBrowserURL()
      tell application "System Events"
          set frontApp to name of first process whose frontmost is true
      end tell

      if frontApp contains "Chrome" then
          tell application "Google Chrome"
              if (count of windows) > 0 then
                  get URL of active tab of front window
              else
                  error "No Chrome windows open"
              end if
          end tell

      else if frontApp contains "Safari" then
          tell application "Safari"
              if (count of windows) > 0 then
                  get URL of front document
              else
                  error "No Safari windows open"
              end if
          end tell

      else if frontApp contains "Firefox" then
          # Firefox用キーボード方式
          tell application "System Events"
              # Cmd+L でアドレスバー選択
              key code 37 using command down
              delay 0.1
              # Cmd+C でコピー
              key code 8 using command down
              delay 0.1
              # Escape で選択解除
              key code 53
          end tell

          set clipboardContent to (the clipboard as string)
          return clipboardContent

      else
          error "No supported browser is active"
      end if
  end getBrowserURL

  try
      set browserURL to getBrowserURL()
      return "SUCCESS|" & browserURL & "|applescript"
  on error errorMessage
      return "ERROR|" & errorMessage & "|applescript"
  end try