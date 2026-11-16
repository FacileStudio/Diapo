//! One page holding a compiled deck: load it, put it at a step, take its picture or its PDF.

use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use url::Url;

use crate::Error;
use crate::browser::{Browser, text};

/// What "loaded" means: the document, its fonts and a deck that says it is ready.
const READY: &str = "document.readyState === 'complete' && document.fonts.status === 'loaded' \
    && !!window.__scene && window.__scene.ready === true";

/// Resolves once two animation frames have run, so the step has been painted.
const PAINTED: &str = "new Promise(done => requestAnimationFrame(() => requestAnimationFrame(() => done(true))))";

/// How long a page gets to become ready.
const READY_TIMEOUT: Duration = Duration::from_secs(20);

/// CSS pixels to an inch, which is how `Page.printToPDF` takes a paper size.
const PIXELS_PER_INCH: f64 = 96.0;

/// One scene of the loaded deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stop {
    /// Its first global step.
    pub first: u32,
    /// How many reveals it has after that.
    pub steps: u32,
}

/// A page attached over CDP, with every request paused for the guard.
pub(crate) struct Page<'a> {
    browser: &'a mut Browser,
    session: String,
    size: (u32, u32),
}

fn bytes(value: &Value, what: &str) -> Result<Vec<u8>, Error> {
    STANDARD.decode(text(value, what)?).map_err(|error| Error::Chrome(format!("bad {what}: {error}")))
}

impl<'a> Page<'a> {
    /// Opens a blank page of `size`.
    pub(crate) fn open(browser: &'a mut Browser, size: (u32, u32)) -> Result<Page<'a>, Error> {
        let target = json!({"url": "about:blank", "width": size.0, "height": size.1});
        let target = text(&browser.call(None, "Target.createTarget", target)?["targetId"], "targetId")?;
        let attach = json!({"targetId": target, "flatten": true});
        let session = text(&browser.call(None, "Target.attachToTarget", attach)?["sessionId"], "sessionId")?;
        let mut page = Page { browser, session, size };
        page.call("Page.enable", json!({}))?;
        page.call("Fetch.enable", json!({"patterns": [{"urlPattern": "*"}]}))?;
        let metrics = json!({"width": size.0, "height": size.1, "deviceScaleFactor": 1, "mobile": false});
        page.call("Emulation.setDeviceMetricsOverride", metrics)?;
        Ok(page)
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.browser.call(Some(&self.session), method, params)
    }

    fn eval(&mut self, expression: &str) -> Result<Value, Error> {
        let params = json!({"expression": expression, "returnByValue": true, "awaitPromise": true});
        let mut answer = self.call("Runtime.evaluate", params)?;
        match answer.get("exceptionDetails") {
            Some(thrown) => {
                let reason = thrown["exception"]["description"].as_str().or(thrown["text"].as_str());
                Err(Error::Deck(reason.unwrap_or("the page threw").to_owned()))
            }
            None => Ok(answer["result"]["value"].take()),
        }
    }

    /// Loads the bundle's `index.html`, waits for the deck and answers its scenes.
    pub(crate) fn load(&mut self) -> Result<Vec<Stop>, Error> {
        let entry = self.browser.guard.root().join("index.html");
        let url = Url::from_file_path(entry).map_err(|()| Error::Job("the bundle path is not absolute".to_owned()))?;
        let answer = self.call("Page.navigate", json!({"url": url.as_str()}))?;
        if let Some(reason) = answer["errorText"].as_str() {
            return Err(Error::Deck(format!("index.html did not load: {reason}")));
        }
        let deadline = Instant::now() + READY_TIMEOUT;
        while self.eval(READY)? != Value::Bool(true) {
            if Instant::now() > deadline {
                return Err(Error::Deck(format!("window.__scene was not ready within {READY_TIMEOUT:?}")));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let scenes = self.eval("window.__scene.scenes.map(scene => [scene.first, scene.steps])")?;
        let stop = |pair: &Value| Some(Stop { first: pair[0].as_u64()? as u32, steps: pair[1].as_u64()? as u32 });
        let stops: Option<Vec<Stop>> = scenes.as_array().map(|list| list.iter().filter_map(stop).collect());
        stops.filter(|stops| !stops.is_empty()).ok_or_else(|| Error::Deck("the deck has no scene".to_owned()))
    }

    /// Draws global step `step`, settled, and waits until it is painted.
    pub(crate) fn step(&mut self, step: u32) -> Result<(), Error> {
        self.eval(&format!("window.__scene.step({step})"))?;
        self.eval(PAINTED).map(|_| ())
    }

    /// The picture of the page as drawn: a PNG at the page size, or a JPEG scaled to `width`.
    pub(crate) fn shot(&mut self, jpeg_width: Option<u32>) -> Result<Vec<u8>, Error> {
        let (width, height) = self.size;
        let scale = jpeg_width.map_or(1.0, |wide| f64::from(wide) / f64::from(width));
        let clip = json!({"x": 0, "y": 0, "width": width, "height": height, "scale": scale});
        let picture = match jpeg_width {
            Some(_) => json!({"format": "jpeg", "quality": 82, "clip": clip}),
            None => json!({"format": "png", "clip": clip}),
        };
        bytes(&self.call("Page.captureScreenshot", picture)?["data"], "screenshot")
    }

    /// The PDF of the page. With `as_printed`, the deck's print stylesheet lays it out: one page
    /// per scene, every step revealed. Without, the page as drawn now, on one page.
    pub(crate) fn pdf(&mut self, as_printed: bool) -> Result<Vec<u8>, Error> {
        let media = if as_printed { "print" } else { "screen" };
        self.call("Emulation.setEmulatedMedia", json!({"media": media}))?;
        let (width, height) = (f64::from(self.size.0), f64::from(self.size.1));
        let mut paper = json!({
            "printBackground": true, "preferCSSPageSize": as_printed,
            "paperWidth": width / PIXELS_PER_INCH, "paperHeight": height / PIXELS_PER_INCH,
            "marginTop": 0, "marginBottom": 0, "marginLeft": 0, "marginRight": 0,
        });
        if !as_printed {
            paper["pageRanges"] = json!("1");
        }
        bytes(&self.call("Page.printToPDF", paper)?["data"], "pdf")
    }
}
