// use crossbeam_channel::unbounded;
use crossbeam_channel;
use notify::{
    self,
    Watcher as NotifyWatcher
};
use std::{
    time::Duration,
    path::PathBuf
};
use super::{
    error::LLResult,
    cse::CSE,
    epw::Epw,
    new_err
};

pub use notify::Error as NotifyError;

pub struct Watcher {
    path: PathBuf,
    cse: CSE,
    watcher: notify::RecommendedWatcher,
    rx: crossbeam_channel::Receiver<Result<notify::Event, notify::Error>>,
    tx: crossbeam_channel::Sender<Result<notify::Event, notify::Error>>
}

impl Watcher {

    pub fn new<P: Into<PathBuf>>(path: P, cse: CSE) -> LLResult<Self> {

        let (tx, rx) = crossbeam_channel::unbounded();
        let w: notify::RecommendedWatcher = notify::Watcher::new(tx.clone(), Duration::from_secs(2))?;

        Ok(Self {
            path: path.into(),
            cse: cse,
            watcher: w,
            rx: rx,
            tx: tx
        })

    }

    pub fn start(&mut self) -> LLResult<()> {

        &self.watcher.watch(&self.path, notify::RecursiveMode::Recursive)?;
        // &self.watcher.configure(notify::Config::OngoingEvents(Some(std::time::Duration::from_millis(500))));

        loop {

            match &self.rx.recv() {

                Ok(event) => {

                    #[cfg(debug_assertions)]
                    println!("{}#{}: {:#?}", std::file!(), std::line!(), event);

                    match event {

                        Ok(evt) => &self.handle_event(evt.clone()),
                        Err(e) => {
                            if format!("{}", e) == "stop" {
                                &self.stop()?;
                                break;
                            } else {
                                // return Err(LLError::new(format!("{}#{}: {}", std::file!(), std::line!(), e)))
                                return Err(new_err!(e))
                            }
                        }

                    }

                },
                Err(e) => {
                    return Err(new_err!(e))
                }

            };

        }

        Ok(())

    }

    pub fn get_tx(&self) -> crossbeam_channel::Sender<Result<notify::Event, notify::Error>> {
        self.tx.clone()
    }

    fn stop(&mut self) -> LLResult<()> {

        println!("Stopping...");

        match &self.watcher.unwatch(&self.path) {
            Ok(v) => Ok(*v),
            Err(e) => Err(new_err!(e))
        }

    }

    fn handle_event(&self, evt: notify::Event) {

        let event = evt.clone();
        let path = &event.paths[0];

        match event.kind {

            notify::EventKind::Create(create_kind) => {

                match create_kind {

                    notify::event::CreateKind::Any => {

                        if path.as_path().is_file() {

                            println!("=> Detected: {}", path.as_path().to_str().unwrap());

                            match &self.handle_file(path) {
                                Ok(p) => println!("=> Success: Saved to {}", p),
                                Err(e) => eprintln!("=> Error: {}", e)
                            };

                        }

                    },

                    notify::event::CreateKind::File => {

                        match &self.handle_file(path) {
                            Ok(p) => println!("=> Success: Saved to {}", p),
                            Err(e) => eprintln!("=> Error: {}", e)
                        };

                    },

                    _ => {}

                };

            },

            _ => {}

        };

    }

    fn handle_file<P: Into<PathBuf>>(&self, path: P) -> LLResult<String> {

        let epw = Epw::from_file(path)?;
        let res = &self.cse.get(epw)?;
        res.save()

    }

}


