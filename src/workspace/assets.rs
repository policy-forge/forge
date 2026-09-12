//! Content-addressed, embedded offline assets with no project data.

pub(crate) const SCRIPT: &str = include_str!("../../ui/workspace.js");
pub(crate) const STYLE: &str = include_str!("../../ui/workspace.css");

pub(crate) fn script_path() -> String {
    format!("/assets/{}.js", crate::hashing::sha256_hex(SCRIPT.as_bytes()))
}
pub(crate) fn style_path() -> String {
    format!("/assets/{}.css", crate::hashing::sha256_hex(STYLE.as_bytes()))
}

pub(crate) fn shell() -> String {
    format!(
        r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="forge-api-major" content="1"><meta name="forge-asset-contract" content="1">
<title>FORGE · Local workspace</title><link rel="stylesheet" href="{}"><script type="module" src="{}"></script></head>
<body><a class="skip" href="#main">Skip to content</a>
<header><strong class="brand">FORGE<span>LOCAL WORKSPACE</span></strong><span id="connection">Locked</span><button id="stop" hidden>Stop workspace</button></header>
<main id="main" tabindex="-1"><section id="unlock-panel" class="unlock">
<p class="eyebrow">YOUR PROJECT. YOUR DECISIONS.</p><h1>Open your workspace.</h1>
<p>Enter the passphrase you set in the terminal. Your files stay on this computer.</p>
<form id="unlock-form"><label for="passphrase">Workspace passphrase</label><input id="passphrase" type="password" maxlength="512" autocomplete="off" required><button type="submit">Unlock workspace</button></form>
<p class="muted">Local unlock provides access to this session. It does not verify reviewer identity.</p></section>
<section id="workspace" hidden><nav aria-label="Workspace views" id="navigation"></nav><div class="content"><div class="view-heading"><h1 id="view-title">Overview</h1><button id="refresh">Refresh</button></div><div id="view"></div></div></section>
<div id="error" role="alert" tabindex="-1" hidden></div><p id="status" role="status" aria-live="polite"></p>
<dialog id="preview-dialog" aria-labelledby="preview-title"><div id="preview-content"></div></dialog>
<dialog id="stop-dialog" aria-labelledby="stop-title"><h2 id="stop-title">Stop this workspace?</h2><p>Unconfirmed form changes will be discarded. Completed writes remain saved.</p><button id="keep-working">Keep working</button><button id="confirm-stop">Stop workspace</button></dialog>
</main><footer>Explicit decisions · Traceable sources · Local files</footer></body></html>"##,
        style_path(),
        script_path()
    )
}
