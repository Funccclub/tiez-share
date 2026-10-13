fn file_transfer_theme_variants(theme: &str) -> (&'static str, &'static str, &'static str) {
    match theme {
        "sticky-note" => (
            r#"
            --bg-body: #fff4b8;
            --bg-panel: rgba(255, 249, 211, 0.9);
            --bg-input: rgba(255, 255, 255, 0.74);
            --bg-button: rgba(255, 255, 255, 0.3);
            --border-dark: #c7a35d;
            --text-primary: #5d4037;
            --text-secondary: #7a5c50;
            --accent-color: #f0a70b;
            --shadow-color: rgba(93, 64, 55, 0.16);
            --font-mono: "Consolas", "Courier New", monospace;
            --content-font-family: "Comic Sans MS", "Chalkboard SE", "Segoe Print", "Microsoft YaHei", sans-serif;
            --radius: 12px;
            --bubble-received-bg: #fffde7;
            --panel-border: 1px solid rgba(199, 163, 93, 0.28);
            --panel-radius: 14px;
            --panel-shadow: 0 10px 22px rgba(93, 64, 55, 0.08);
            --input-border: 1px dashed rgba(240, 167, 11, 0.7);
            --input-radius: 8px;
            --input-shadow: none;
            --button-border: none;
            --button-radius: 999px;
            --button-shadow: none;
            --button-active-transform: scale(0.98);
            --button-active-shadow: none;
            --button-active-filled-shadow: 0 10px 22px rgba(240, 167, 11, 0.24);
            --send-button-background: rgba(240, 167, 11, 0.18);
            --send-button-color: var(--text-primary);
            --send-button-border: 1px dashed rgba(240, 167, 11, 0.44);
            --send-button-shadow: 0 10px 22px rgba(240, 167, 11, 0.18);
            "#,
            r#"
            --bg-body: #242427;
            --bg-panel: rgba(47, 47, 51, 0.9);
            --bg-input: rgba(255, 255, 255, 0.08);
            --bg-button: rgba(255, 255, 255, 0.06);
            --border-dark: rgba(255, 212, 77, 0.24);
            --text-primary: #f2eee3;
            --text-secondary: #d0c7b5;
            --accent-color: #ffd44d;
            --shadow-color: rgba(0, 0, 0, 0.28);
            --bubble-received-bg: #3a3a3c;
            --panel-border: 1px solid rgba(255, 212, 77, 0.16);
            --panel-shadow: 0 14px 28px rgba(0, 0, 0, 0.22);
            --input-border: 1px dashed rgba(255, 212, 77, 0.42);
            --send-button-background: rgba(255, 212, 77, 0.14);
            --send-button-border: 1px dashed rgba(255, 212, 77, 0.3);
            --send-button-shadow: 0 10px 22px rgba(0, 0, 0, 0.2);
            "#,
            r#"
            body.theme-sticky-note {
                background-color: var(--bg-body);
                background-image: linear-gradient(180deg, rgba(255, 255, 255, 0.4), rgba(255, 243, 188, 0.22));
                background-size: 100% 100%;
            }
            body.theme-sticky-note.dark-mode {
                background-image: linear-gradient(180deg, rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.22));
            }
            @media (prefers-color-scheme: dark) {
                body.theme-sticky-note:not(.light-mode) {
                    background-image: linear-gradient(180deg, rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.22));
                }
            }
            .theme-sticky-note .avatar,
            .theme-sticky-note .bubble,
            .theme-sticky-note .text-input,
            .theme-sticky-note .retro-btn {
                border-style: solid !important;
            }
            .theme-sticky-note .message.received .bubble {
                transform: rotate(-0.5deg);
                box-shadow: 2px 2px 0 var(--shadow-color);
            }
            .theme-sticky-note .message.sent .bubble {
                transform: rotate(0.8deg);
                box-shadow: -2px 2px 0 var(--shadow-color);
            }
            "#,
        ),
        "paper" => (
            r#"
            --bg-body: #f4ecd8;
            --bg-window: #fdf6e3;
            --bg-panel: rgba(255, 253, 247, 0.78);
            --bg-input: #ffffff;
            --bg-button: rgba(139, 90, 43, 0.08);
            --border-dark: #d5c4a1;
            --text-primary: #3c3836;
            --text-secondary: #7c6f64;
            --accent-color: #8b5a2b;
            --shadow-color: rgba(60, 56, 54, 0.12);
            --font-mono: "Courier New", Courier, monospace;
            --content-font-family: "Georgia", "STSong", "SimSun", "Songti SC", serif;
            --radius: 2px;
            --bubble-received-bg: #fffdf7;
            --panel-border: 1px solid #d5c4a1;
            --panel-radius: 2px;
            --panel-shadow: 0 2px 10px rgba(0, 0, 0, 0.03);
            --input-border: 1px solid #d5c4a1;
            --input-radius: 2px;
            --input-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.03);
            --button-border: 1px solid rgba(139, 90, 43, 0.3);
            --button-radius: 4px;
            --send-button-background: rgba(139, 90, 43, 0.12);
            --send-button-color: var(--text-primary);
            --send-button-border: 1px solid rgba(139, 90, 43, 0.28);
            --send-button-shadow: 0 4px 10px rgba(139, 90, 43, 0.12);
            "#,
            r#"
            --bg-body: #282828;
            --bg-panel: rgba(50, 48, 47, 0.82);
            --bg-input: #1d2021;
            --bg-button: rgba(213, 196, 161, 0.1);
            --border-dark: #504945;
            --text-primary: #ebdbb2;
            --text-secondary: #a89984;
            --accent-color: #d79921;
            --shadow-color: rgba(0, 0, 0, 0.3);
            --bubble-received-bg: #32302f;
            --panel-border: 1px solid #504945;
            --input-border: 1px solid #504945;
            --send-button-background: rgba(215, 153, 33, 0.16);
            --send-button-color: var(--text-primary);
            --send-button-border: 1px solid rgba(215, 153, 33, 0.26);
            --send-button-shadow: 0 4px 10px rgba(0, 0, 0, 0.18);
            "#,
            r#"
            body.theme-paper {
                background-image: linear-gradient(rgba(139, 90, 43, 0.06) 1px, transparent 1px);
                background-size: 100% 1.65em;
            }
            body.theme-paper::before {
                content: "";
                position: fixed;
                inset: 0;
                background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
                opacity: 0.04;
                pointer-events: none;
                z-index: 9999;
            }
            @media (prefers-color-scheme: dark) {
                body.theme-paper:not(.light-mode) {
                    background-image: linear-gradient(rgba(213, 196, 161, 0.04) 1px, transparent 1px);
                }
            }
            "#,
        ),
        "mica" => (
            r#"
            --bg-body: #f3f3f3;
            --bg-panel: rgba(255, 255, 255, 0.48);
            --bg-input: rgba(255, 255, 255, 0.74);
            --bg-button: rgba(255, 255, 255, 0.46);
            --border-dark: rgba(128, 128, 128, 0.18);
            --text-primary: #1a2435;
            --text-secondary: #607188;
            --accent-color: #4f7dff;
            --shadow-color: rgba(15, 23, 42, 0.1);
            --font-mono: system-ui, -apple-system, sans-serif;
            --radius: 12px;
            --bubble-received-bg: rgba(255, 255, 255, 0.9);
            --panel-border: 1px solid rgba(255, 255, 255, 0.3);
            --send-button-background: #4f7dff;
            --send-button-color: #ffffff;
            "#,
            r#"
            --bg-body: #1a1a1a;
            --bg-panel: rgba(36, 36, 36, 0.52);
            --bg-input: rgba(255, 255, 255, 0.12);
            --bg-element: rgba(45, 45, 45, 0.65);
            --bg-button: rgba(255, 255, 255, 0.05);
            --text-primary: #e8e8e8;
            --text-secondary: #a8a8a8;
            --accent-color: #4f7dff;
            --border-dark: rgba(255, 255, 255, 0.08);
            --bubble-received-bg: rgba(40, 40, 40, 0.9);
            --send-button-background: #4f7dff;
            --send-button-color: #ffffff;
            "#,
            r#"
            .theme-mica header, .theme-mica footer {
                backdrop-filter: blur(20px) saturate(180%);
                -webkit-backdrop-filter: blur(20px) saturate(180%);
            }
            .theme-mica .message.received .bubble {
                backdrop-filter: blur(10px);
                -webkit-backdrop-filter: blur(10px);
            }
            .theme-mica .retro-btn.send-btn {
                background: var(--send-button-background);
                color: var(--send-button-color);
                border: none;
                box-shadow: 0 4px 12px rgba(79, 125, 255, 0.3);
            }
            "#,
        ),
        "acrylic" => (
            r#"
            --bg-body: #f3f3f3;
            --bg-panel: rgba(255, 255, 255, 0.34);
            --bg-input: rgba(255, 255, 255, 0.52);
            --bg-button: rgba(255, 255, 255, 0.2);
            --border-dark: rgba(255, 255, 255, 0.32);
            --text-primary: #162234;
            --text-secondary: #607188;
            --accent-color: #4f7dff;
            --shadow-color: rgba(15, 23, 42, 0.08);
            --font-mono: system-ui, -apple-system, sans-serif;
            --radius: 10px;
            --bubble-received-bg: rgba(255, 255, 255, 0.4);
            --send-button-background: #4f7dff;
            --send-button-color: #ffffff;
            "#,
            r#"
            --bg-body: #141414;
            --bg-panel: rgba(28, 28, 28, 0.38);
            --bg-input: rgba(255, 255, 255, 0.12);
            --bg-button: rgba(255, 255, 255, 0.06);
            --border-dark: rgba(255, 255, 255, 0.12);
            --text-primary: #e8e8e8;
            --text-secondary: #b2b2b2;
            --accent-color: #4f7dff;
            --bubble-received-bg: rgba(45, 45, 45, 0.42);
            --send-button-background: #4f7dff;
            --send-button-color: #ffffff;
            "#,
            r#"
            .theme-acrylic header, .theme-acrylic footer {
                backdrop-filter: blur(30px) saturate(145%);
                -webkit-backdrop-filter: blur(30px) saturate(145%);
            }
            .theme-acrylic .message.received .bubble {
                backdrop-filter: blur(15px);
                -webkit-backdrop-filter: blur(15px);
            }
            .theme-acrylic .retro-btn.send-btn {
                background: var(--send-button-background);
                color: var(--send-button-color);
                border: none;
                box-shadow: 0 4px 12px rgba(79, 125, 255, 0.3);
            }
            "#,
        ),
        _ => (
            r#"
            --bg-body: #dcdcdc;
            --bg-panel: #f3f3f3;
            --bg-input: #ffffff;
            --bg-button: #e0e0e0;
            --border-dark: #373737;
            --text-primary: #373737;
            --text-secondary: #707070;
            --accent-color: #487bdb;
            --shadow-color: #373737;
            --font-mono: "Courier New", Courier, monospace;
            --content-font-family: var(--font-mono);
            --radius: 0px;
            --bubble-received-bg: #ffffff;
            --panel-border: 2px solid var(--border-dark);
            --panel-radius: 4px;
            --panel-shadow: 2px 2px 0 0 var(--shadow-color);
            --input-border: 3px solid var(--border-dark);
            --input-radius: 0;
            --input-shadow: inset 4px 4px 0 rgba(0, 0, 0, 0.1);
            --button-border: 2px solid var(--border-dark);
            --button-radius: 0;
            --button-shadow: 2px 2px 0 0 var(--shadow-color);
            --button-active-transform: translate(2px, 2px);
            --button-active-shadow: 0 0 0 0 var(--shadow-color);
            --button-active-filled-shadow: inset 2px 2px 0 rgba(0, 0, 0, 0.2);
            --send-button-background: var(--accent-color);
            --send-button-color: #ffffff;
            --send-button-border: 2px solid var(--border-dark);
            --send-button-shadow: inset 2px 2px 0 rgba(0, 0, 0, 0.2);
            "#,
            r#"
            --bg-body: #121212;
            --bg-panel: #1e1e1e;
            --bg-input: #202020;
            --bg-button: #333333;
            --border-dark: #000000;
            --text-primary: #e0e0e0;
            --text-secondary: #a0a0a0;
            --accent-color: #5a8dee;
            --shadow-color: #000000;
            --bubble-received-bg: #1e1e1e;
            --panel-border: 2px solid #000000;
            --panel-shadow: 2px 2px 0 0 #000000;
            --input-border: 2px solid #000000;
            --input-shadow: inset 2px 2px 0 0 rgba(0,0,0,0.5);
            --button-border: 2px solid #000000;
            --button-shadow: 2px 2px 0 0 #000000;
            --send-button-background: #000000;
            --send-button-color: #ffffff;
            --send-button-border: 2px solid #000000;
            --send-button-shadow: none;
            "#,
            "",
        ),
    }
}

fn file_transfer_theme_css(theme: &str, color_mode: &str) -> String {
    let (light, dark, extra) = file_transfer_theme_variants(theme);
    let dark_css = match color_mode {
        "dark" => format!(":root {{{dark}}}"),
        "system" => format!("@media (prefers-color-scheme: dark) {{ :root {{{dark}}} }}"),
        _ => String::new(),
    };

    format!(":root {{{light}}}\n{dark_css}\n{extra}")
}

pub fn render_index(theme: &str, color_mode: &str, logo_base64: &str, title: &str) -> String {
    let theme_css = file_transfer_theme_css(theme, color_mode);
    let mode_class = match color_mode {
        "dark" => "dark-mode",
        "light" => "light-mode",
        _ => "",
    };

    format!(
        r#"
<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover, interactive-widget=resizes-content">
    <title>{title}</title>
    <style>
        * {{ box-sizing: border-box; -webkit-tap-highlight-color: transparent; }}
        
        :root {{
            --bg-body: #dcdcdc;
            --bg-panel: #f3f3f3;
            --border-dark: #373737;
            --text-primary: #373737;
            --accent-color: #487bdb;
            --shadow-color: #373737;
            --font-mono: "Courier New", Courier, monospace;
            --radius: 0px;
            --bubble-received-bg: #ffffff;
            --app-height: 100vh;
        }}

        @media (prefers-color-scheme: dark) {{
            .theme-mica, .theme-acrylic {{
                --bg-body: #101010;
                --bg-panel: #1c1c1c;
                --border-dark: rgba(255,255,255,0.08); /* Fainter border */
                --text-primary: #e0e0e0;
                --shadow-color: rgba(0,0,0,0.5);
            }}
            :root:not(.theme-mica):not(.theme-acrylic) {{
                --bg-body: #121212; /* Slightly cooler black */
                --bg-panel: #1e1e1e;
                --border-dark: #2a2a2a; /* Dark gray instead of pure black for borders */
                --text-primary: #e0e0e0; 
                --shadow-color: #000000;
                --bubble-received-bg: var(--bg-panel);
            }}
        }}

        /* Theme mica / acrylic overrides */
        .theme-mica, .theme-acrylic {{
            --bg-body: #f3f3f3;
            --bg-panel: #ffffff;
            --border-dark: rgba(0,0,0,0.1);
            --text-primary: #333;
            --shadow-color: rgba(0,0,0,0.1);
            --font-mono: "Segoe UI", system-ui, -apple-system, sans-serif;
            --radius: 12px;
            --bubble-received-bg: rgba(255, 255, 255, 0.9);
        }}

        @media (prefers-color-scheme: dark) {{
            .theme-mica, .theme-acrylic {{
                --bg-body: #1a1a1a;
                --bg-panel: #2a2a2a;
                --text-primary: #e0e0e0;
                --bubble-received-bg: rgba(40, 40, 40, 0.9);
            }}
        }}

        html, body {{
            height: 100%;
            width: 100%;
            margin: 0;
            padding: 0;
            overflow: hidden;
            position: fixed;
            top: 0;
            left: 0;
        }}

        body {{
            background-color: var(--bg-body);
            color: var(--text-primary);
            font-family: var(--content-font-family, var(--font-mono));
            display: flex;
            flex-direction: column;
            height: var(--app-height);
            transition: background 0.3s;
        }}

        header {{
            height: 60px;
            background: var(--bg-panel);
            border-bottom: 2px solid var(--border-dark);
            display: flex; align-items: center; justify-content: center;
            padding: 0 16px; position: relative;
            flex-shrink: 0;
            z-index: 10;
        }}
        .theme-mica header, .theme-acrylic header {{
            background: rgba(255,255,255,0.7); backdrop-filter: blur(20px);
            border-bottom: 1px solid rgba(0,0,0,0.1);
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica header, .theme-acrylic header {{ background: rgba(30,30,30,0.7); }}
        }}

        h1 {{ font-size: 18px; font-weight: 900; margin: 0; letter-spacing: -0.5px; }}
        
        .header-status {{
            position: absolute; left: 16px; top: 50%; transform: translateY(-50%);
            display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: bold;
        }}
        .status-dot {{ width: 8px; height: 8px; background: #4caf50; border-radius: 50%; box-shadow: 0 0 5px #4caf50; }}
        .header-me {{
            position: absolute; right: 16px; top: 50%; transform: translateY(-50%);
            font-size: 12px; font-weight: bold; cursor: pointer;
            max-width: 32%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
            opacity: 0.85;
        }}

        #chat-box {{
            flex: 1; overflow-y: auto; padding: 16px;
            display: flex; flex-direction: column; gap: 16px;
            scroll-behavior: smooth;
            padding-bottom: 40px;
        }}

        .timestamp {{
            font-size: 11px; text-align: center; opacity: 0.6;
            margin: 8px 0; font-weight: bold;
        }}

        .message {{ display: flex; gap: 10px; max-width: 90%; }}
        .message.received {{ align-self: flex-start; }}
        .message.sent {{ align-self: flex-end; flex-direction: row-reverse; }}

        .avatar {{
            width: 36px; height: 36px; background: #fff;
            border: 2px solid var(--border-dark);
            display: flex; align-items: center; justify-content: center;
            font-weight: bold; font-size: 18px; flex-shrink: 0;
            box-shadow: 2px 2px 0 var(--shadow-color);
            border-radius: var(--radius); overflow: hidden;
        }}
        .avatar img {{ width: 100%; height: 100%; object-fit: cover; }}
        
        .theme-mica .avatar, .theme-acrylic .avatar {{
            box-shadow: none; border-width: 1px;
        }}

        .bubble {{
            padding: 10px 14px;
            background: #fff;
            border: 2px solid var(--border-dark);
            box-shadow: 3px 3px 0 var(--shadow-color);
            font-size: 14px; line-height: 1.5;
            word-break: break-all;
            position: relative;
            border-radius: var(--radius);
        }}
        .message.received .bubble {{ background: var(--bubble-received-bg); }}
        .message.sent .bubble {{ background: var(--accent-color); color: #fff; border-color: var(--border-dark); }}

        /* 系统公告小提醒（toast）：加入/离开等事件不占聊天气泡，
           在窗口顶部短暂浮现后自动消失 */
        .toast-container {{
            position: fixed;
            top: 68px;
            left: 0;
            right: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
            pointer-events: none;
            z-index: 9990;
        }}
        .toast-item {{
            background: var(--bg-panel);
            border: 1px solid var(--border-dark);
            border-left: 4px solid var(--accent-color);
            color: var(--text-primary);
            font-size: 12px;
            font-family: var(--content-font-family, var(--font-mono));
            padding: 8px 16px;
            border-radius: var(--radius);
            box-shadow: 0 4px 14px var(--shadow-color);
            max-width: 82vw;
            text-align: center;
            word-break: break-word;
            opacity: 0;
            transform: translateY(-8px);
            transition: opacity 0.25s ease, transform 0.25s ease;
        }}
        .toast-item.show {{
            opacity: 1;
            transform: translateY(0);
        }}
        .toast-item.hide {{
            opacity: 0;
            transform: translateY(-8px);
        }}
        
        /* Mica Style Bubbles */
        .theme-mica .bubble, .theme-acrylic .bubble {{
            box-shadow: 0 2px 10px var(--shadow-color);
            border: 1px solid var(--border-dark);
            border-radius: 12px;
        }}
        .theme-mica .message.received .bubble, .theme-acrylic .message.received .bubble {{
             background: var(--bubble-received-bg);
             backdrop-filter: blur(10px);
        }}

        /* Triangle tail for retro style only */
        :root:not(.theme-mica):not(.theme-acrylic) .message.received .bubble::after {{
            content: ''; position: absolute; left: -10px; top: 10px;
            width: 0; height: 0; border: 5px solid transparent;
            border-right-color: var(--bubble-received-bg);
        }}
        :root:not(.theme-mica):not(.theme-acrylic) .message.received .bubble::before {{
            content: ''; position: absolute; left: -13px; top: 9px;
            width: 0; height: 0; border: 6px solid transparent;
            border-right-color: var(--border-dark);
        }}
        :root:not(.theme-mica):not(.theme-acrylic) .message.sent .bubble::after {{
            content: ''; position: absolute; right: -10px; top: 10px;
            width: 0; height: 0; border: 5px solid transparent;
            border-left-color: var(--accent-color);
        }}
        :root:not(.theme-mica):not(.theme-acrylic) .message.sent .bubble::before {{
            content: ''; position: absolute; right: -13px; top: 9px;
            width: 0; height: 0; border: 6px solid transparent;
            border-left-color: var(--border-dark);
        }}
        
        /* Mica doesn't use these triangles */
        .theme-mica .message.received .bubble::before,
        .theme-acrylic .message.received .bubble::before {{ border-right-color: var(--border-dark); }}
        .theme-mica .message.received .bubble::after,
        .theme-acrylic .message.received .bubble::after {{ border-right-color: var(--bubble-received-bg); }}
        
        @media (prefers-color-scheme: dark) {{
            .theme-mica .message.received .bubble, .theme-acrylic .message.received .bubble {{ background: var(--bubble-received-bg); }}
        }}

        .theme-mica .message.sent .bubble::before,
        .theme-acrylic .message.sent .bubble::before {{ display: none; }}
        .theme-mica .message.sent .bubble::after,
        .theme-acrylic .message.sent .bubble::after {{ 
            border-left-color: var(--accent-color) !important; 
            right: -7px; 
            bottom: 10px;
        }}

        
        /* File Card */
        .file-card {{ display: flex; align-items: center; gap: 12px; }}
        .file-icon {{ font-size: 24px; flex-shrink: 0; }}
        .file-info {{ display: flex; flex-direction: column; min-width: 0; overflow: hidden; }}
        .file-name {{ font-weight: 700; font-family: var(--font-mono); font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 160px; }}
        .file-size {{ font-size: 11px; font-family: var(--font-mono); opacity: 0.8; margin-top: 2px; }}

        /* 复制按钮：绝对定位贴在气泡右下角，鼠标悬停整条消息时浮现。
           触屏没有 hover 状态，用 @media (hover: none) 常驻半透明显示。 */
        .bubble.has-copy {{ padding-right: 34px; }}
        .copy-btn {{
            position: absolute; right: 6px; bottom: 6px;
            width: 24px; height: 24px; padding: 0; margin: 0;
            display: flex; align-items: center; justify-content: center;
            font: inherit; color: inherit; background: transparent;
            border: 1px solid transparent; border-radius: 6px;
            cursor: pointer; opacity: 0;
            transition: opacity .15s ease, background .15s ease;
        }}
        .message:hover .copy-btn, .copy-btn:focus-visible {{ opacity: .65; }}
        .copy-btn:hover {{ opacity: 1; background: rgba(127,127,127,.18); }}
        .copy-btn.copied {{ opacity: 1; }}
        /* 触屏没有 hover 就没有浮现过程，必须常驻可见。
           0.4 太淡——sticky-note 主题下发送气泡是白字，本来对比度就低。 */
        @media (hover: none) {{ .copy-btn {{ opacity: .65; }} }}

        /* Image Preview */
        .img-preview {{
            max-width: 100%;
            width: auto;
            max-height: 400px;
            display: block;
            border: 1px solid rgba(0,0,0,0.1);
            margin-bottom: 6px;
            border-radius: 8px;
            object-fit: contain;
        }}
        .video-preview {{
            width: 100%;
            max-width: 100%;
            min-height: 120px;
            max-height: 400px;
            display: block;
            margin-bottom: 6px;
            border-radius: 8px;
            background: #000;
        }}

        .progress-wrapper {{ margin-top: 8px; border-top: 1px dashed rgba(255,255,255,0.3); padding-top: 4px; }}
        .progress-bar {{ width: 100%; height: 4px; background: rgba(0,0,0,0.1); border-radius: 2px; overflow: hidden; margin-top: 4px; }}
        .progress-inner {{ height: 100%; background: var(--accent-color); width: 0%; transition: width 0.2s; }}

        footer {{
            padding: 10px 16px;
            padding-bottom: calc(10px + env(safe-area-inset-bottom));
            background: var(--bg-panel);
            border-top: 2px solid var(--border-dark);
            display: flex; gap: 12px; align-items: flex-end;
            flex-shrink: 0;
            z-index: 10;
        }}
        .theme-mica footer, .theme-acrylic footer {{
            background: rgba(255,255,255,0.7); backdrop-filter: blur(20px);
            border-top: 1px solid rgba(0,0,0,0.1);
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica footer, .theme-acrylic footer {{ background: rgba(30,30,30,0.7); }}
        }}

        /* ===== 左侧文件区 ===== */
        #main {{ flex: 1; display: flex; min-height: 0; }}
        #chat-column {{ flex: 1; display: flex; flex-direction: column; min-width: 0; min-height: 0; }}

        #file-panel {{
            width: 248px; flex-shrink: 0; min-height: 0;
            display: flex; flex-direction: column;
            background: var(--bg-panel);
            border-right: 2px solid var(--border-dark);
        }}
        .theme-mica #file-panel, .theme-acrylic #file-panel {{
            background: rgba(255,255,255,0.55); backdrop-filter: blur(20px);
            border-right: 1px solid rgba(0,0,0,0.1);
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica #file-panel, .theme-acrylic #file-panel {{ background: rgba(30,30,30,0.6); }}
        }}

        #file-panel-head {{
            display: flex; align-items: center; gap: 8px; flex-shrink: 0;
            padding: 8px 10px; font-size: 12px; font-weight: 900;
            border-bottom: 2px solid var(--border-dark);
        }}
        .theme-mica #file-panel-head, .theme-acrylic #file-panel-head {{
            border-bottom: 1px solid rgba(0,0,0,0.1);
        }}
        #file-meta {{ margin-left: auto; font-size: 10px; font-weight: 700; opacity: 0.6; }}
        .fp-btn {{ cursor: pointer; opacity: 0.6; font-size: 14px; padding: 0 4px; }}
        .fp-btn:hover {{ opacity: 1; }}

        #file-list {{
            flex: 1; overflow-y: auto; overflow-x: hidden; padding: 8px;
            display: flex; flex-direction: column; gap: 8px;
        }}
        #file-list.drag-over {{ outline: 2px dashed var(--accent-color); outline-offset: -6px; }}
        #file-list .empty {{ font-size: 11px; opacity: 0.55; text-align: center; padding: 20px 8px; line-height: 1.7; }}

        .file-item {{
            display: flex; align-items: center; gap: 8px; padding: 8px;
            border: var(--panel-border, 2px solid var(--border-dark));
            border-radius: var(--radius); background: var(--bg-input);
            cursor: pointer; transition: border-color 0.15s;
        }}
        .file-item:hover {{ border-color: var(--accent-color); }}
        .fi-icon {{ font-size: 17px; flex-shrink: 0; }}
        .fi-body {{ flex: 1; min-width: 0; }}
        .fi-name {{ font-size: 12px; font-weight: 700; line-height: 1.3; word-break: break-all; }}
        .fi-info {{ font-size: 10px; opacity: 0.6; margin-top: 3px; }}
        .fi-del {{ flex-shrink: 0; cursor: pointer; opacity: 0.4; font-size: 13px; padding: 2px 3px; }}
        .fi-del:hover {{ opacity: 1; }}

        /* ===== 拖拽上传覆盖层 ===== */
        #drop-overlay {{
            position: fixed; inset: 0; z-index: 9998; display: none;
            align-items: center; justify-content: center;
            background: rgba(0,0,0,0.45); color: #fff;
            font-weight: 900; font-size: 16px; pointer-events: none;
        }}
        #drop-overlay.show {{ display: flex; }}
        #drop-overlay .dz {{
            border: 3px dashed #fff; border-radius: 16px; padding: 26px 34px;
            text-align: center; line-height: 1.8;
        }}

        .files-btn {{ width: 40px; font-size: 18px; }}

        /* 桌面端可折叠；窄屏改成左侧抽屉 */
        @media (min-width: 721px) {{
            body.panel-collapsed #file-panel {{ display: none; }}
        }}
        @media (max-width: 720px) {{
            #file-panel {{
                position: fixed; top: 60px; bottom: 0; left: 0;
                width: 80%; max-width: 320px; z-index: 30;
                transform: translateX(-100%); transition: transform 0.25s ease;
                box-shadow: 4px 0 18px rgba(0,0,0,0.28);
            }}
            body.panel-open #file-panel {{ transform: translateX(0); }}
        }}
        
        .retro-btn {{
            background: var(--bg-button);
            border: var(--button-border, 2px solid var(--border-dark));
            box-shadow: var(--button-shadow, 2px 2px 0 0 var(--shadow-color));
            display: flex; align-items: center; justify-content: center;
            cursor: pointer; color: var(--text-primary);
            transition: all 0.1s;
            height: 40px;
            flex-shrink: 0;
            border-radius: var(--button-radius, var(--radius));
        }}
        .retro-btn:active {{
            transform: var(--button-active-transform, translate(2px, 2px));
            box-shadow: var(--button-active-shadow, 0 0 0 0 var(--shadow-color));
        }}
        
        .theme-mica .retro-btn, .theme-acrylic .retro-btn {{
            box-shadow: none; border-width: 1px;
            background: rgba(255,255,255,0.5);
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica .retro-btn, .theme-acrylic .retro-btn {{ background: rgba(255,255,255,0.1); }}
        }}

        .add-btn {{ width: 40px; font-size: 24px; font-weight: 900; }}
        .send-btn {{
            min-width: 72px;
            padding: 0 16px;
            background: var(--send-button-background, var(--accent-color));
            color: var(--send-button-color, #ffffff);
            border: var(--send-button-border, var(--button-border, 2px solid var(--border-dark)));
            box-shadow: var(--send-button-shadow, var(--button-active-filled-shadow, 2px 2px 0 0 var(--shadow-color)));
        }}
        
        .text-input {{ 
            flex: 1; height: 40px; min-height: 40px; max-height: 100px; padding: 10px 12px;
            background: var(--bg-input); border: var(--input-border, 2px solid var(--border-dark));
            box-shadow: var(--input-shadow, inset 2px 2px 0 rgba(0,0,0,0.1));
            font-size: 14px; font-family: var(--content-font-family, var(--font-mono));
            color: var(--text-primary); outline: none;
            border-radius: var(--input-radius, var(--radius)); -webkit-appearance: none;
            resize: none; overflow-y: auto; line-height: 20px;
        }}
        .theme-mica .text-input, .theme-acrylic .text-input {{ box-shadow: none; border-width: 1px; }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica .text-input, .theme-acrylic .text-input {{ background: rgba(255,255,255,0.05); color: #fff; }}

            /* Retro Dark Mode Overrides */
            :root:not(.theme-mica):not(.theme-acrylic) .retro-btn {{
                background: #333;
                border-color: #000;
                color: #e0e0e0;
                box-shadow: 2px 2px 0 0 #000;
            }}
            :root:not(.theme-mica):not(.theme-acrylic) .retro-btn:active {{
                box-shadow: none;
                transform: translate(2px, 2px);
            }}
            :root:not(.theme-mica):not(.theme-acrylic) .retro-btn.send-btn {{
                background: #000;
                color: #fff;
                border-color: #000;
            }}
            :root:not(.theme-mica):not(.theme-acrylic) .text-input {{
                background: #202020;
                color: #e0e0e0;
                border-color: #000;
                box-shadow: inset 2px 2px 0 0 rgba(0,0,0,0.5);
            }}
        }}
        
        .expand-btn {{
            width: 30px; height: 30px; display: none; align-items: center; justify-content: center;
            position: absolute; right: 5px; bottom: 5px;
            background: var(--bg-body); border: 1px solid var(--border-dark);
            border-radius: 4px; cursor: pointer; z-index: 5;
            color: var(--text-primary);
        }}

        /* Fullscreen Editor */
        #fs-editor {{
            position: fixed; top: 0; left: 0; width: 100%; height: 100%;
            background: var(--bg-body); z-index: 1000;
            display: none; flex-direction: column;
            padding: 16px;
        }}
        .theme-mica #fs-editor, .theme-acrylic #fs-editor {{
             background: rgba(255,255,255,0.95); backdrop-filter: blur(20px);
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica #fs-editor, .theme-acrylic #fs-editor {{ background: rgba(20,20,20,0.95); }}
            
            /* Retro Dark Mode Overrides for Fullscreen Editor */
            :root:not(.theme-mica):not(.theme-acrylic) #fs-textarea {{
                background: #202020;
                color: #e0e0e0;
                border: 2px solid #000;
            }}
        }}

        #fs-textarea {{
            flex: 1; width: 100%; border: 2px solid var(--border-dark);
            padding: 16px; font-size: 16px; font-family: var(--font-mono);
            background: #fff; color: var(--text-primary); margin-bottom: 16px;
            border-radius: var(--radius); resize: none; outline: none;
        }}
        .theme-mica #fs-textarea, .theme-acrylic #fs-textarea {{
            background: rgba(255,255,255,0.5); border-width: 1px; box-shadow: none;
        }}
        @media (prefers-color-scheme: dark) {{
            .theme-mica #fs-textarea, .theme-acrylic #fs-textarea {{ background: rgba(255,255,255,0.05); color: #fff; }}
        }}

        .fs-toolbar {{ display: flex; justify-content: flex-end; gap: 12px; }}
        {theme_css}
    </style>
</head>
<body class="theme-{theme} {mode_class}">
    <header>
        <div class="header-status">
            <div class="status-dot"></div>
            <span id="device-count">连接中…</span>
        </div>
        <h1>{title}</h1>
        <div class="header-me" id="me-name" title="点击修改本机名称">…</div>
    </header>

    <div id="main">
        <aside id="file-panel">
            <div id="file-panel-head">
                <span>📁 文件区</span>
                <span id="file-meta">—</span>
                <span class="fp-btn" id="file-refresh" title="重新扫描文件列表">⟳</span>
            </div>
            <div id="file-list"><div class="empty">加载中…</div></div>
        </aside>

        <div id="chat-column">
    <div id="chat-box">
        <div class="timestamp">SYS: <span id="time-now"></span></div>
        <div class="message received">
            <div class="avatar"><img src="{logo_base64}" onerror="this.innerText='T'"></div>
            <div class="bubble">
                <div style="font-weight:900; margin-bottom:4px">SYSTEM READY</div>
                局域网中转已就绪：发送的文字、图片和文件，所有在线设备都会收到。
            </div>
        </div>
    </div>
    
    <!-- Full Screen Editor Modal -->
    <div id="fs-editor">
        <div style="font-weight:bold; margin-bottom:8px; display:flex; justify-content:space-between; align-items:center">
            <span>FULL SCREEN EDIT</span>
            <span onclick="closeFullscreen()" style="cursor:pointer; padding:4px;">✕</span>
        </div>
        <textarea id="fs-textarea" placeholder="输入内容..." enterkeyhint="send"></textarea>
        <div class="fs-toolbar">
            <button class="retro-btn" onclick="closeFullscreen()">CANCEL</button>
            <button class="retro-btn send-btn" onclick="sendFullscreen()">SEND</button>
        </div>
    </div>

    <footer>
        <label for="file-input" class="retro-btn add-btn">+</label>
        <button class="retro-btn files-btn" id="files-toggle" title="显示/隐藏文件区">📁</button>
        <div style="position:relative; flex:1; display:flex;">
            <textarea class="text-input" id="text-input" placeholder="输入文字... 回车发送" rows="1" enterkeyhint="send"></textarea>
            <div class="expand-btn" id="expand-btn" onclick="openFullscreen()">⤢</div>
        </div>
        <button class="retro-btn send-btn" id="send-btn">SEND</button>
        <input type="file" id="file-input" multiple style="display:none">
    </footer>
        </div><!-- /#chat-column -->

    <!-- 系统公告 toast 容器（加入/离开提醒，非聊天消息） -->
    <div id="toast-container" class="toast-container"></div>

    </div><!-- /#main -->

    <!-- 拖拽上传提示（整页都是放置区） -->
    <div id="drop-overlay">
        <div class="dz">
            📥 松手即可上传
            <div style="font-size:12px; font-weight:400; opacity:.85; margin-top:6px;">
                文件保存到服务端，并出现在左侧文件区
            </div>
        </div>
    </div>

    <!-- Fullscreen Image Overlay -->
    <div id="img-overlay" style="display:none; position:fixed; top:0; left:0; width:100%; height:100%; background:rgba(0,0,0,0.9); backdrop-filter:blur(5px); z-index:9999; align-items:center; justify-content:center; flex-direction:column;">
        <div style="position:absolute; top:20px; right:20px; color:white; font-size:24px; cursor:pointer; padding:10px;" onclick="closeOverlay()">✕</div>
        <img id="overlay-img" style="max-width:95%; max-height:90%; object-fit:contain; border-radius:4px; box-shadow:0 0 20px rgba(0,0,0,0.5);">
    </div>

    <script>
        const fileInput = document.getElementById('file-input');
        const textInput = document.getElementById('text-input');
        const sendBtn = document.getElementById('send-btn');
        const chatBox = document.getElementById('chat-box');
        
        const now = new Date();
        document.getElementById('time-now').innerText = `${{now.getHours().toString().padStart(2,'0')}}:${{now.getMinutes().toString().padStart(2,'0')}}`;
        
        let lastId = 0;
        let isUploading = false;
        const deviceId = localStorage.getItem('tiez_device_id') || ('d-' + Math.random().toString(36).substr(2, 9));
        localStorage.setItem('tiez_device_id', deviceId);

        // 每个浏览器就是聊天室里的一个成员：名字可用 ?name=xxx 指定，
        // 之后保存在 localStorage，点右上角可随时改名。
        function detectDeviceName() {{
            const fromUrl = new URLSearchParams(window.location.search).get('name');
            if (fromUrl && fromUrl.trim()) {{
                const n = fromUrl.trim().slice(0, 24);
                localStorage.setItem('tiez_device_name', n);
                return n;
            }}
            const saved = localStorage.getItem('tiez_device_name');
            if (saved) return saved;
            const ua = navigator.userAgent;
            if (/iPhone/i.test(ua)) return 'iPhone';
            if (/iPad/i.test(ua)) return 'iPad';
            if (/Android/i.test(ua)) return 'Android';
            if (/Macintosh/i.test(ua)) return 'Mac';
            if (/Windows/i.test(ua)) return 'Windows PC';
            if (/Linux/i.test(ua)) return 'Linux';
            return '设备';
        }}
        let deviceName = detectDeviceName();
        
        const TIEZ_LOGO = "{logo_base64}";
        const pendingUploads = new Map(); // filename -> [elements]
        const renderedIds = new Set();

        const meNameEl = document.getElementById('me-name');
        meNameEl.innerText = deviceName;
        meNameEl.onclick = () => {{
            const next = prompt('本机显示名称（其他设备会看到）', deviceName);
            if (!next || !next.trim()) return;
            deviceName = next.trim().slice(0, 24);
            localStorage.setItem('tiez_device_name', deviceName);
            meNameEl.innerText = deviceName;
            if (socket && socket.readyState === 1) {{
                socket.send(JSON.stringify({{ type: 'identity', device_id: deviceId, device_name: deviceName }}));
            }}
        }};

        function scrollToBottom() {{
            chatBox.scrollTop = chatBox.scrollHeight;
        }}

        function syncViewportMetrics() {{
            const vv = window.visualViewport;
            if (!vv) {{
                document.documentElement.style.setProperty('--app-height', `${{window.innerHeight}}px`);
                return;
            }}

            // Pull the height from the visual viewport (excludes keyboard)
            document.documentElement.style.setProperty('--app-height', `${{vv.height}}px`);
            
            // Keep the fixed body aligned with the visual viewport's top/left
            document.body.style.transform = `translate(${{vv.offsetLeft}}px, ${{vv.offsetTop}}px)`;
            
            // Prevent the browser from scrolling the layout viewport away from origin
            if (window.scrollY !== 0 || window.scrollX !== 0) {{
                window.scrollTo(0, 0);
            }}
        }}

        if (window.visualViewport) {{
            window.visualViewport.addEventListener('resize', syncViewportMetrics);
            window.visualViewport.addEventListener('scroll', syncViewportMetrics);
        }}
        window.addEventListener('resize', syncViewportMetrics);
        // Also sync on focus/blur to handle various keyboard states
        document.addEventListener('focusin', () => setTimeout(syncViewportMetrics, 50));
        document.addEventListener('focusout', () => setTimeout(syncViewportMetrics, 50));
        syncViewportMetrics();

        function escapeHTML(str) {{
            if (!str) return '';
            return str.replace(/[&<>"']/g, function(m) {{
                return {{
                    '&': '&amp;',
                    '<': '&lt;',
                    '>': '&gt;',
                    '"': '&quot;',
                    "'": '&#39;'
                }}[m];
            }});
        }}

        function normalizeFileName(name) {{
            if (!name) return '';
            const base = name.split('/').pop().split('\\').pop();
            const m = base.match(/^\d{{8,}}_(?:[0-9a-f]{{16,}}-[0-9a-f]+_)?(.+)$/);
            return escapeHTML(m ? m[1] : base);
        }}

        // 左侧文件区用 innerText 渲染，不能再经过 escapeHTML，
        // 否则文件名里的 & 会原样显示成 &amp;
        function displayName(name) {{
            if (!name) return '';
            const base = name.split('/').pop().split('\\').pop();
            const m = base.match(/^\d{{8,}}_(?:[0-9a-f]{{16,}}-[0-9a-f]+_)?(.+)$/);
            return m ? m[1] : base;
        }}
        function extractNameFromContent(content, file_path) {{
            if (content && content.startsWith('/download/') && content.includes('?name=')) {{
                const idx = content.indexOf('?name=');
                if (idx !== -1) {{
                    try {{ return decodeURIComponent(content.slice(idx + 6)); }} catch (e) {{ return content; }}
                }}
            }}
            return file_path || content || '';
        }}
        function addPendingUpload(fileName, el) {{
            const list = pendingUploads.get(fileName) || [];
            list.push(el);
            pendingUploads.set(fileName, list);
        }}
        function takePendingUpload(maybeName) {{
            for (const [name, list] of pendingUploads.entries()) {{
                if (maybeName.endsWith(name) && list.length > 0) {{
                    const el = list.shift();
                    if (list.length === 0) pendingUploads.delete(name);
                    return el;
                }}
            }}
            return null;
        }}

        // 复制按钮的图标。用内联 SVG 而不是 emoji / 符号字形：
        // 路由器上访问的设备什么系统都有，字形覆盖不可控。
        const COPY_ICON = '<svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="5.5" y="5.5" width="8" height="9" rx="1.5"></rect><path d="M10.5 3.2V3a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v6.5a1 1 0 0 0 1 1h.2"></path></svg>';
        const COPIED_ICON = '<svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 8.5l3.5 3.5 7.5-8"></path></svg>';

        // 页面通常是用 http://路由器IP:12345 打开的，那不是安全上下文，
        // navigator.clipboard 不存在，必须保留 execCommand 兜底。
        async function copyToClipboard(text) {{
            try {{
                if (window.isSecureContext && navigator.clipboard) {{
                    await navigator.clipboard.writeText(text);
                    return true;
                }}
            }} catch (e) {{ /* 权限被拒等情况落到下面的兜底 */ }}
            const ta = document.createElement('textarea');
            ta.value = text;
            ta.setAttribute('readonly', '');
            ta.style.cssText = 'position:fixed;top:0;left:0;width:1px;height:1px;padding:0;border:0;opacity:0;';
            document.body.appendChild(ta);
            ta.focus();
            ta.select();
            ta.setSelectionRange(0, ta.value.length);
            let ok = false;
            try {{ ok = document.execCommand('copy'); }} catch (e) {{ ok = false; }}
            ta.remove();
            return ok;
        }}

        // 文字消息复制正文；图片/视频/文件复制可用的下载链接（补成绝对地址，
        // 直接复制 /download/xxx 别人打开没意义）。data: 和 blob: 复制出来没法用，
        // 不给按钮——上传中的占位气泡正好走这条路。
        function copyTargetOf(content, msgType, file_path) {{
            const link = content.startsWith('/download/') ? content : (file_path || '');
            if (link && !/^(data|blob):/.test(link)) {{
                return /^https?:/i.test(link) ? link : window.location.origin + link;
            }}
            if (msgType === 'image' || msgType === 'video' || msgType === 'file' || file_path) return '';
            return content || '';
        }}

        function attachCopyButton(bubble, text) {{
            if (!text) return;
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'copy-btn';
            btn.innerHTML = COPY_ICON;
            btn.title = '复制';
            btn.setAttribute('aria-label', '复制消息');
            // 气泡本身挂了 onclick 跳转下载，复制按钮必须拦住冒泡，
            // 否则点"复制"会变成下载文件。
            btn.addEventListener('click', async (e) => {{
                e.preventDefault();
                e.stopPropagation();
                if (await copyToClipboard(text)) {{
                    btn.innerHTML = COPIED_ICON;
                    btn.classList.add('copied');
                    setTimeout(() => {{
                        btn.innerHTML = COPY_ICON;
                        btn.classList.remove('copied');
                    }}, 1200);
                }}
            }});
            bubble.appendChild(btn);
            bubble.classList.add('has-copy');
        }}

        function createMessageElement(direction, content, senderName, msgType, file_path) {{
            const div = document.createElement('div');
            div.className = `message ${{direction}}`;
            
            let bubbleContent = escapeHTML(content);
            if (msgType === 'image' || (content.match(/\.(jpg|jpeg|png|gif|webp)$/i) && file_path)) {{
                const useContent = content.startsWith('data:') || content.startsWith('/download/') || content.startsWith('http');
                const src = escapeHTML(useContent ? content : (file_path || content));
                bubbleContent = `<img src="${{src}}" class="img-preview" onclick="openOverlay('${{src}}')">`;
            }} else if (msgType === 'video') {{
                const useContent = content.startsWith('/download/') || content.startsWith('http');
                const src = escapeHTML(useContent ? content : (file_path || content));
                bubbleContent = `<video class="video-preview" controls src="${{src}}"></video>`;
            }} else if (msgType === 'file' || file_path) {{
                 const rawName = extractNameFromContent(content, file_path);
                 const fileName = normalizeFileName(rawName);
                 bubbleContent = `
                    <div class="file-card">
                        <div class="file-icon">📄</div>
                        <div class="file-info">
                            <span class="file-name">${{fileName}}</span>
                            <span class="file-size">DOWNLOAD</span>
                        </div>
                    </div>
                 `;
            }}

            const escapedSenderName = escapeHTML(senderName);
            div.innerHTML = `
                ${{direction === 'received' ? (() => {{
                    const name = (senderName || '').trim();
                    const lower = name.toLowerCase();
                    const isPc = name === '电脑' || name === 'PC' || lower === 'pc' || lower === 'tiez';
                    if (isPc) {{
                        return `<div class="avatar"><img src="${{TIEZ_LOGO}}" alt="TieZ"></div>`;
                    }}
                    return `<div class="avatar">${{name ? escapeHTML(name[0]) : '?'}}</div>`;
                }})() : ''}}
                <div class="bubble">
                    ${{escapedSenderName && escapedSenderName !== 'System' ? `<div style="font-size:10px; opacity:0.6; margin-bottom:2px">${{escapedSenderName}}</div>` : ''}}
                    ${{bubbleContent}}
                </div>
            `;
            
            const bubble = div.querySelector('.bubble');
            const downloadUrl = content.startsWith('/download/') ? content : file_path;
            if (downloadUrl && msgType !== 'image' && msgType !== 'video') {{
                bubble.style.cursor = 'pointer';
                bubble.onclick = () => window.location.href = downloadUrl;
            }}
            attachCopyButton(bubble, copyTargetOf(content, msgType, file_path));

            return div;
        }}

        function openOverlay(src) {{
            document.getElementById('overlay-img').src = src;
            document.getElementById('img-overlay').style.display = 'flex';
        }}
        function closeOverlay() {{
            document.getElementById('img-overlay').style.display = 'none';
        }}

        function openFullscreen() {{
            document.getElementById('fs-textarea').value = textInput.value;
            document.getElementById('fs-editor').style.display = 'flex';
            document.getElementById('fs-textarea').focus();
        }}
        function closeFullscreen() {{
            document.getElementById('fs-editor').style.display = 'none';
        }}
        function sendFullscreen() {{
            const val = document.getElementById('fs-textarea').value;
            if (val.trim()) {{
                textInput.value = val;
                sendBtn.click();
            }}
            closeFullscreen();
        }}

        // Adjust textarea height
        textInput.addEventListener('input', function() {{
            this.style.height = '40px';
            const newHeight = Math.min(this.scrollHeight, 100);
            this.style.height = newHeight + 'px';
            document.getElementById('expand-btn').style.display = newHeight > 50 ? 'flex' : 'none';
        }});

        // 回车直接发送，Shift+回车换行。
        // e.isComposing 必须挡掉：中文输入法组字时按回车是"选词"，
        // 旧版 WebKit 上 isComposing 还没置位，用 keyCode 229 兜一层。
        function isEnterToSend(e) {{
            if (e.key !== 'Enter' || e.shiftKey || e.ctrlKey || e.metaKey) return false;
            return !e.isComposing && e.keyCode !== 229;
        }}

        textInput.addEventListener('keydown', (e) => {{
            if (!isEnterToSend(e)) return;
            e.preventDefault();
            sendBtn.click();
        }});

        // 全屏编辑器同理，否则两个输入框行为不一致
        document.getElementById('fs-textarea').addEventListener('keydown', (e) => {{
            if (!isEnterToSend(e)) return;
            e.preventDefault();
            sendFullscreen();
        }});

        textInput.addEventListener('focus', () => {{
            setTimeout(() => {{
                syncViewportMetrics();
                scrollToBottom();
            }}, 80);
        }});

        window.addEventListener('resize', syncViewportMetrics);
        window.addEventListener('orientationchange', syncViewportMetrics);
        if (window.visualViewport) {{
            window.visualViewport.addEventListener('resize', syncViewportMetrics);
            window.visualViewport.addEventListener('scroll', syncViewportMetrics);
        }}
        syncViewportMetrics();

        // 系统公告（加入/离开等）：服务端用 msg_type="notice" 广播，
        // 这里渲染成顶部小 toast 提醒，不进聊天消息流
        function renderToast(text) {{
            const container = document.getElementById('toast-container');
            if (!container) return;
            const el = document.createElement('div');
            el.className = 'toast-item';
            el.textContent = text;
            container.appendChild(el);
            requestAnimationFrame(() => requestAnimationFrame(() => el.classList.add('show')));
            const remove = () => {{
                el.classList.remove('show');
                el.classList.add('hide');
                setTimeout(() => el.remove(), 300);
            }};
            // 3.5 秒后自动消失；点击提前关闭
            el.addEventListener('click', remove);
            setTimeout(remove, 3500);
        }}

        function isSystemNotice(msg) {{
            return msg.msg_type === 'notice' || (msg.sender_id === 'server' && msg.id === 0);
        }}

        // 历史消息和实时消息都走这里渲染。
        // 判定"是不是我发的"看 sender_id，而不是 direction：中继端把所有浏览器
        // 发来的内容都记成 direction=in，只按 direction 判断会让每个人的消息
        // 都显示在自己右侧，聊天室就没法用了。
        function renderMessage(msg) {{
            // 系统公告走 toast，不渲染成聊天气泡
            if (isSystemNotice(msg)) {{
                if (msg.content) renderToast(msg.content);
                return;
            }}
            if (msg.id) {{
                if (renderedIds.has(msg.id)) return;
                renderedIds.add(msg.id);
                if (msg.id > lastId) lastId = msg.id;
            }}
            const mine = msg.sender_id === deviceId;
            const side = mine ? 'sent' : 'received';
            const name = mine ? 'You' : (msg.sender_name || '设备');

            // 自己刚上传的文件：用正式气泡替换上传中的占位气泡
            if (mine && (msg.msg_type === 'file' || msg.msg_type === 'image' || msg.msg_type === 'video')) {{
                const rawName = extractNameFromContent(msg.content, msg.file_path);
                const pending = takePendingUpload(normalizeFileName(rawName));
                if (pending) {{
                    const replacement = createMessageElement('sent', msg.content, 'You', msg.msg_type, msg.file_path);
                    if (msg.id) replacement.dataset.id = msg.id;
                    pending.replaceWith(replacement);
                    scrollToBottom();
                    return;
                }}
            }}

            const el = createMessageElement(side, msg.content, name, msg.msg_type, msg.file_path);
            if (msg.id) el.dataset.id = msg.id;
            chatBox.appendChild(el);
            scrollToBottom();
        }}

        function updateDeviceRoster(devices) {{
            const el = document.getElementById('device-count');
            el.innerText = `在线 ${{devices.length}}`;
            el.title = devices.map(d => d.name).join('\n');
        }}

        // 打开页面时补齐历史记录，否则新连上的设备看不到之前的对话
        async function loadHistory() {{
            try {{
                const res = await fetch('/poll?last_id=0');
                if (!res.ok) return;
                const msgs = await res.json();
                for (const m of msgs) renderMessage(m);
            }} catch (e) {{ /* 中继未就绪时忽略，WS 会自行重连 */ }}
        }}

        // WebSocket Setup
        let socket;
        function connectWS() {{
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            socket = new WebSocket(`${{protocol}}//${{window.location.host}}/ws`);

            socket.onopen = () => {{
                socket.send(JSON.stringify({{ type: 'identity', device_id: deviceId, device_name: deviceName }}));
                console.log('WS Connected');
            }};

            socket.onmessage = (e) => {{
                const msg = JSON.parse(e.data);
                if (msg.type === 'devices_update') {{
                    updateDeviceRoster(msg.devices || []);
                    return;
                }}
                if (msg.type === 'files_changed') {{
                    // 有人上传或删除了文件：同步左侧文件区；
                    // 被删掉的文件，聊天里对应的消息也一起移除
                    for (const id of (msg.removed_message_ids || [])) {{
                        const gone = chatBox.querySelector(`.message[data-id="${{id}}"]`);
                        if (gone) gone.remove();
                    }}
                    loadFiles();
                    return;
                }}
                renderMessage(msg);
            }};

            socket.onclose = () => setTimeout(connectWS, 3000);
        }}
        connectWS();
        loadHistory();

        sendBtn.onclick = async () => {{
            const text = textInput.value.trim();
            if (!text || isUploading) return;
            
            textInput.value = '';
            textInput.style.height = '40px';
            document.getElementById('expand-btn').style.display = 'none';

            try {{
                await fetch('/send-text', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ content: text, sender_id: deviceId, sender_name: deviceName }})
                }});
            }} catch(e) {{ alert('Send failed'); }}
        }};

        fileInput.onchange = async () => {{
            if (!fileInput.files.length || isUploading) return;
            const files = Array.from(fileInput.files);
            fileInput.value = '';
            
            for(const file of files) {{
                await uploadFile(file);
            }}
        }};


        // Incremental SHA-256 also works on plain HTTP LAN pages, where
        // crypto.subtle is unavailable. Memory use stays at one 512 KiB chunk.
        class UploadSHA256 {{
            constructor() {{
                this.h = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
                this.buffer = new Uint8Array(64); this.used = 0; this.length = 0;
            }}
            block(bytes) {{
                const k = [0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
                const r = (x,n) => (x >>> n) | (x << (32-n));
                const w = new Int32Array(64);
                for(let i=0;i<16;i++) w[i]=(bytes[i*4]<<24)|(bytes[i*4+1]<<16)|(bytes[i*4+2]<<8)|bytes[i*4+3];
                for(let i=16;i<64;i++) {{
                    const x=w[i-15],y=w[i-2];
                    w[i]=(w[i-16]+(r(x,7)^r(x,18)^(x>>>3))+w[i-7]+(r(y,17)^r(y,19)^(y>>>10)))|0;
                }}
                let [a,b,c,d,e,f,g,h]=this.h;
                for(let i=0;i<64;i++) {{
                    const t1=(h+(r(e,6)^r(e,11)^r(e,25))+((e&f)^(~e&g))+k[i]+w[i])|0;
                    const t2=((r(a,2)^r(a,13)^r(a,22))+((a&b)^(a&c)^(b&c)))|0;
                    h=g;g=f;f=e;e=(d+t1)|0;d=c;c=b;b=a;a=(t1+t2)|0;
                }}
                [a,b,c,d,e,f,g,h].forEach((v,i)=>this.h[i]=(this.h[i]+v)|0);
            }}
            update(bytes) {{
                this.length += bytes.length;
                let i=0;
                if(this.used) {{
                    const n=Math.min(64-this.used,bytes.length);
                    this.buffer.set(bytes.subarray(0,n),this.used);this.used+=n;i=n;
                    if(this.used===64) {{ this.block(this.buffer);this.used=0; }}
                }}
                for(;i+64<=bytes.length;i+=64) this.block(bytes.subarray(i,i+64));
                if(i<bytes.length) {{ this.buffer.set(bytes.subarray(i));this.used=bytes.length-i; }}
                return this;
            }}
            digest() {{
                const bits=this.length*8;
                const pad=new Uint8Array(this.used<56?64-this.used:128-this.used);
                pad[0]=0x80;
                const view=new DataView(pad.buffer);
                view.setUint32(pad.length-8,Math.floor(bits/0x100000000));
                view.setUint32(pad.length-4,bits>>>0);
                this.update(pad);
                return this.h.map(v=>(v>>>0).toString(16).padStart(8,'0')).join('');
            }}
        }}

        async function uploadFile(file) {{
            isUploading = true;
            const uploadSenderId = deviceId, uploadSenderName = deviceName;
            const isImage = file.type.startsWith('image/');
            const isVideo = file.type.startsWith('video/');
            const msgType = isVideo ? 'video' : (isImage ? 'image' : 'file');
            const previewUrl = (isImage || isVideo) ? URL.createObjectURL(file) : file.name;
            const el = createMessageElement('sent', previewUrl, 'You', msgType, undefined);
            el.dataset.fileName = file.name;
            el.dataset.pending = 'true';
            addPendingUpload(file.name, el);
            const progressWrapper = document.createElement('div');
            progressWrapper.className = 'progress-wrapper';
            progressWrapper.innerHTML = `<div style="font-size:10px">校验中</div><div class="progress-bar"><div class="progress-inner"></div></div>`;
            el.querySelector('.bubble').appendChild(progressWrapper);
            chatBox.appendChild(el);
            scrollToBottom();
            try {{
                const CHUNK_SIZE = 1024 * 512;
                const totalChunks = Math.max(1, Math.ceil(file.size / CHUNK_SIZE));
                const uploadId = Date.now().toString(36)+'-'+Math.random().toString(36).slice(2);
                const whole = new UploadSHA256(), hashes = [];
                for(let i=0;i<totalChunks;i++) {{
                    const bytes=new Uint8Array(await file.slice(i*CHUNK_SIZE,(i+1)*CHUNK_SIZE).arrayBuffer());
                    whole.update(bytes);
                    hashes.push(new UploadSHA256().update(bytes).digest());
                    progressWrapper.querySelector('div').innerText='校验中 '+Math.round((i+1)/totalChunks*100)+'%';
                    await new Promise(resolve=>setTimeout(resolve,0));
                }}
                const fileHash=whole.digest();
                for(let i=0;i<totalChunks;i++) {{
                    const formData=new FormData();
                    formData.append('file',file.slice(i*CHUNK_SIZE,(i+1)*CHUNK_SIZE));
                    formData.append('metadata',JSON.stringify({{upload_id:uploadId,chunk_index:i,
                        total_chunks:totalChunks,file_name:file.name,sender_id:uploadSenderId,
                        sender_name:uploadSenderName,total_size:file.size,content_type:file.type,
                        chunk_sha256:hashes[i],file_sha256:fileHash}}));
                    for(let attempt=0;;attempt++) {{
                        const controller=new AbortController();
                        const timer=setTimeout(()=>controller.abort(),30000);
                        try {{
                            const res=await fetch('/upload-chunk',{{method:'POST',body:formData,signal:controller.signal}});
                            if(!res.ok) {{
                                const error=new Error(await res.text());
                                error.permanent=res.status<500 && ![408,429].includes(res.status);
                                throw error;
                            }}
                            break;
                        }} catch(error) {{
                            if(error.permanent || attempt>=3) throw error;
                            progressWrapper.querySelector('div').innerText='正在重试 '+(attempt+1)+'/3';
                            await new Promise(resolve=>setTimeout(resolve,500*2**attempt));
                        }} finally {{ clearTimeout(timer); }}
                    }}
                    const percent=Math.round((i+1)/totalChunks*100);
                    progressWrapper.querySelector('.progress-inner').style.width=percent+'%';
                    progressWrapper.querySelector('div').innerText=percent+'%';
                }}
                const badge=document.createElement('span');badge.style.color='#4caf50';badge.textContent=' ✓';
                el.querySelector('.bubble').appendChild(badge);
            }} catch(error) {{
                el.dataset.pending='false';
                const list=pendingUploads.get(file.name)||[],idx=list.indexOf(el);
                if(idx>=0) list.splice(idx,1);
                if(list.length===0) pendingUploads.delete(file.name);
                const badge=document.createElement('span');badge.style.color='#e0533d';badge.textContent=' ✗ 上传失败';
                el.querySelector('.bubble').appendChild(badge);
                alert('上传失败：'+file.name+'\n'+error.message);
            }} finally {{
                progressWrapper.remove();
                isUploading=false;
            }}
        }}

        // ===================== 左侧文件区 =====================
        const fileListEl = document.getElementById('file-list');
        const fileMetaEl = document.getElementById('file-meta');
        const dropOverlay = document.getElementById('drop-overlay');

        function formatSize(bytes) {{
            if (!bytes) return '0 B';
            if (bytes < 1024) return bytes + ' B';
            const units = ['KB', 'MB', 'GB', 'TB'];
            let v = bytes / 1024, i = 0;
            while (v >= 1024 && i < units.length - 1) {{ v /= 1024; i++; }}
            return (v >= 10 ? v.toFixed(0) : v.toFixed(1)) + ' ' + units[i];
        }}

        function formatWhen(ms) {{
            if (!ms) return '';
            const d = new Date(ms), now = new Date();
            const p = n => String(n).padStart(2, '0');
            const hm = `${{p(d.getHours())}}:${{p(d.getMinutes())}}`;
            return d.toDateString() === now.toDateString()
                ? hm
                : `${{d.getMonth() + 1}}/${{d.getDate()}} ${{hm}}`;
        }}

        function fileIcon(name) {{
            const n = (name || '').toLowerCase();
            if (/\.(png|jpe?g|gif|webp|bmp|svg|ico|heic)$/.test(n)) return '🖼️';
            if (/\.(mp4|mov|mkv|avi|webm|flv|wmv|m4v)$/.test(n)) return '🎬';
            if (/\.(mp3|wav|flac|m4a|aac|ogg|opus)$/.test(n)) return '🎵';
            if (/\.(zip|rar|7z|tar|gz|xz|bz2)$/.test(n)) return '🗜️';
            if (/\.pdf$/.test(n)) return '📕';
            if (/\.(apk|exe|msi|deb|ipk|dmg|iso)$/.test(n)) return '📦';
            if (/\.(sh|bat|cmd|ps1|py|js|ts|rs|go|c|h|cpp|java|php|html?|css)$/.test(n)) return '📜';
            return '📄';
        }}

        async function loadFiles() {{
            try {{
                const res = await fetch('/files');
                if (!res.ok) return;
                const data = await res.json();
                renderFileList(data);
            }} catch (e) {{ /* 网络抖动就下次刷新自愈 */ }}
        }}

        function renderFileList(data) {{
            const files = data.files || [];
            fileMetaEl.innerText = files.length
                ? `${{files.length}} 个 · ${{formatSize(data.total_size || 0)}}`
                : '空';

            if (!files.length) {{
                fileListEl.innerHTML = '<div class="empty">还没有文件<br>把文件拖进这个页面就能上传</div>';
                return;
            }}

            fileListEl.innerHTML = '';
            for (const f of files) {{
                const item = document.createElement('div');
                item.className = 'file-item';
                item.title = '点击下载';

                const icon = document.createElement('div');
                icon.className = 'fi-icon';
                icon.innerText = fileIcon(f.name);

                const body = document.createElement('div');
                body.className = 'fi-body';
                const nm = document.createElement('div');
                nm.className = 'fi-name';
                nm.innerText = displayName(f.name);
                const info = document.createElement('div');
                info.className = 'fi-info';
                info.innerText = `${{formatSize(f.size)}} · ${{formatWhen(f.modified)}}`;
                body.appendChild(nm);
                body.appendChild(info);

                const del = document.createElement('div');
                del.className = 'fi-del';
                del.innerText = '🗑';
                del.title = '删除这个文件';
                del.onclick = (ev) => {{ ev.stopPropagation(); deleteFile(f); }};

                item.appendChild(icon);
                item.appendChild(body);
                item.appendChild(del);
                // 图片/视频的下载响应是 inline（聊天里的预览需要），
                // 所以不能直接跳转，否则点图片会变成"在浏览器里打开图片"。
                // 同源 URL 加 download 属性可以强制走下载。
                item.onclick = () => {{
                    const a = document.createElement('a');
                    a.href = f.url;
                    a.download = displayName(f.name);
                    document.body.appendChild(a);
                    a.click();
                    a.remove();
                    if (window.matchMedia('(max-width: 720px)').matches) {{
                        document.body.classList.remove('panel-open'); // 窄屏点完收起抽屉
                    }}
                }};
                fileListEl.appendChild(item);
            }}
        }}

        async function deleteFile(f) {{
            const name = displayName(f.name);
            const ok = confirm('删除「' + name + '」？\n\n' +
                '文件会从服务器上删除，聊天记录里对应的那条文件消息也会一起移除，无法恢复。');
            if (!ok) return;
            try {{
                const res = await fetch('/files/' + encodeURIComponent(f.token), {{ method: 'DELETE' }});
                if (!res.ok) {{ alert('删除失败（HTTP ' + res.status + '）'); return; }}
                await loadFiles();
            }} catch (e) {{
                alert('删除失败：' + e.message);
            }}
        }}

        function togglePanel() {{
            if (window.matchMedia('(max-width: 720px)').matches) {{
                document.body.classList.toggle('panel-open');       // 窄屏：左侧抽屉
            }} else {{
                const collapsed = document.body.classList.toggle('panel-collapsed');
                localStorage.setItem('tiez_panel_collapsed', collapsed ? '1' : '0');
            }}
        }}

        document.getElementById('file-refresh').onclick = loadFiles;
        document.getElementById('files-toggle').onclick = togglePanel;
        if (localStorage.getItem('tiez_panel_collapsed') === '1') {{
            document.body.classList.add('panel-collapsed');
        }}

        // ===================== 拖拽上传（整页都可放） =====================
        let dragDepth = 0;
        function draggingFiles(e) {{
            return !!(e.dataTransfer && Array.from(e.dataTransfer.types || []).indexOf('Files') !== -1);
        }}

        window.addEventListener('dragenter', e => {{
            if (!draggingFiles(e)) return;
            dragDepth++;
            dropOverlay.classList.add('show');
            fileListEl.classList.add('drag-over');
        }});
        window.addEventListener('dragover', e => {{
            if (!draggingFiles(e)) return;
            e.preventDefault();
            e.dataTransfer.dropEffect = 'copy';
        }});
        window.addEventListener('dragleave', e => {{
            if (!draggingFiles(e)) return;
            dragDepth = Math.max(0, dragDepth - 1);
            if (dragDepth === 0) {{
                dropOverlay.classList.remove('show');
                fileListEl.classList.remove('drag-over');
            }}
        }});
        // 拖到页面空白处时，别让浏览器直接把文件当成页面打开
        document.addEventListener('dragover', e => e.preventDefault());
        document.addEventListener('drop', e => e.preventDefault());

        window.addEventListener('drop', async e => {{
            e.preventDefault();
            const files = Array.from((e.dataTransfer && e.dataTransfer.files) || []);
            dragDepth = 0;
            dropOverlay.classList.remove('show');
            fileListEl.classList.remove('drag-over');
            if (!files.length) return;

            for (const file of files) {{
                await uploadFile(file);
            }}
            await loadFiles();
        }});

        loadFiles();

    </script>
</body>
</html>
    "#,
        theme = theme,
        mode_class = mode_class,
        theme_css = theme_css,
        logo_base64 = logo_base64,
        title = title
    )
}
