//! The morning refusal replay (§437.5): every file of the fuse fuzzer's corpus
//! run through its own harness, and every refusal the harness meets counted
//! into a group.
//!
//! The fuzzer drops a refusal once it has counted it, and keeps the input that
//! reached it. So nothing new is collected during the night; the corpus is
//! replayed the morning after, and the refusals come back out of it.
//!
//! **One process, with a time limit per input.** Two of the night's inputs of
//! 2026-10-11 did not return in 120 s (Q1721). The files are shared out among
//! worker threads, and a worker whose file runs past the limit is left behind
//! with it and replaced, so one hung input does not stop the rest. The process
//! ending is what stops a worker left behind.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use rcad_predicates::{Decision, Policy};
use rcad_topo::Error;

use crate::fusing::{self, WayRound, WhichFuse};
use crate::{numbers_as_n, on_one_line};

/// What replaying a whole corpus found.
#[derive(Clone, Debug)]
pub struct CorpusReplay {
    /// One per kind of refusal, the most refusals first.
    pub groups: Vec<RefusalGroup>,
    /// How many files were handed in, every one below included.
    pub files: usize,
    /// How many files met at least one refusal.
    pub files_that_refused: usize,
    /// Files whose replay panicked, each with what the panic said and where.
    pub panicked: Vec<(PathBuf, String)>,
    /// Files whose replay had not returned when its time ran out.
    pub hung: Vec<PathBuf>,
    /// Files that could not be read, each with why.
    pub unreadable: Vec<(PathBuf, String)>,
    /// How long one file was given before it was called hung.
    pub time_limit: Duration,
}

impl CorpusReplay {
    /// How many refusals there were in all.
    #[must_use]
    pub fn refusals(&self) -> usize {
        self.groups.iter().map(|group| group.refusals).sum()
    }
}

/// One kind of refusal, and how often the corpus met it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusalGroup {
    /// The `Error` variant, with the variant of the error it wraps where it
    /// wraps one: `Invalid(DynamicRangeTooWide)`.
    pub variant: String,
    /// For `Undecidable`, the predicates that gave up, or
    /// [`THE_SETTLED_DECISIONS_ADDED_UP_PAST_THE_BUDGET`] where none did. For
    /// every other variant, what it says, with every number written `N`.
    pub key: String,
    /// How many refusals landed here.
    pub refusals: usize,
    /// How many files met at least one of them.
    pub files: usize,
    /// How many of them were under `Policy::certified()`.
    pub certified: usize,
    /// How many were under a tolerant policy.
    pub tolerant: usize,
    /// How many came from the first call of a fuse.
    pub first_way_round: usize,
    /// How many came from the second call, with the two sides swapped.
    pub other_way_round: usize,
    /// The first file, in the order they were handed in, that met one.
    pub sample: PathBuf,
}

/// The key of an `Undecidable` none of whose records is still undecided.
///
/// `Model::fuse` refuses that way when every decision was settled but their
/// wobble added up past what the policy allows (`fuse/mod.rs`, after the
/// arrangement). No predicate gave up there, so naming the last one in the log
/// would name one that has nothing to do with the refusal (§437.4).
pub const THE_SETTLED_DECISIONS_ADDED_UP_PAST_THE_BUDGET: &str =
    "the settled decisions added up past the budget";

/// How long one file of the night's corpus is given by default. The slowest
/// file that returned on 2026-10-11 took 65.2 s (§437.9).
pub const SECONDS_ONE_FILE_IS_GIVEN: u64 = 120;

/// Replay every file through the fuse harness, `workers` at a time, and group
/// every refusal they meet.
///
/// A file that does not return within `time_limit` is counted as hung and its
/// worker is left running; the caller ends the process to stop it.
///
/// # Panics
///
/// If no thread can be started to replay on.
#[must_use]
pub fn every_refusal_in(files: Vec<PathBuf>, workers: usize, time_limit: Duration) -> CorpusReplay {
    let earlier_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        WHAT_THE_LAST_PANIC_SAID.with(|said| *said.borrow_mut() = info.to_string());
    }));
    let replayed = each_in_parallel(files, workers, time_limit, what_one_file_did);
    panic::set_hook(earlier_hook);
    grouped(replayed, time_limit)
}

/// [`every_refusal_in`]'s groups as tab-separated text, header first, for
/// `fuzz-triage` to write to the night's `fuzz-refusals.tsv`.
#[must_use]
pub fn as_refusals_tsv(replay: &CorpusReplay) -> String {
    let mut out = String::from(
        "variant\tkey\trefusals\tfiles\tcertified\ttolerant\tfirst_way_round\tother_way_round\tsample\n",
    );
    for group in &replay.groups {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            group.variant,
            group.key,
            group.refusals,
            group.files,
            group.certified,
            group.tolerant,
            group.first_way_round,
            group.other_way_round,
            group.sample.display()
        );
    }
    out
}

/// Every file the replay could not finish, as tab-separated text, header
/// first: `fault` (`panicked`, `hung` or `unreadable`), `file`, and `said`,
/// which for a panic is its place and its words.
#[must_use]
pub fn as_faults_tsv(replay: &CorpusReplay) -> String {
    let mut out = String::from("fault\tfile\tsaid\n");
    for (file, said) in &replay.panicked {
        let _ = writeln!(out, "panicked\t{}\t{said}", file.display());
    }
    for file in &replay.hung {
        let _ = writeln!(
            out,
            "hung\t{}\tdid not return in {} s",
            file.display(),
            replay.time_limit.as_secs()
        );
    }
    for (file, why) in &replay.unreadable {
        let _ = writeln!(out, "unreadable\t{}\t{why}", file.display());
    }
    out
}

thread_local! {
    /// What the last panic on this thread said, and where. The hook is the
    /// whole process's, and each worker reads only its own thread's slot, so
    /// two files panicking at once on two workers do not swap messages.
    static WHAT_THE_LAST_PANIC_SAID: RefCell<String> = const { RefCell::new(String::new()) };
}

/// What one file did.
enum WhatOneFileDid {
    Unreadable(String),
    /// A refusal met before a panic was met all the same, so it is kept.
    Replayed {
        refusals: Vec<Refusal>,
        panicked: Option<String>,
    },
}

/// One refusal, already keyed.
struct Refusal {
    variant: String,
    key: String,
    which: WhichFuse,
}

fn what_one_file_did(path: &Path) -> WhatOneFileDid {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(why) => return WhatOneFileDid::Unreadable(why.to_string()),
    };
    let mut refusals = Vec::new();
    let replayed = panic::catch_unwind(AssertUnwindSafe(|| {
        fusing::two_models_split_and_fused_noting_refusals(&bytes, &mut |why, which| {
            refusals.push(keyed(why, which));
        })
    }));
    let panicked = replayed
        .err()
        .map(|_| WHAT_THE_LAST_PANIC_SAID.with(|said| on_one_line(&said.borrow())));
    WhatOneFileDid::Replayed { refusals, panicked }
}

/// A refusal's group: for `Undecidable`, the predicates of the records whose
/// outcome is still undecided, since the log also holds every decision a
/// tolerant policy settled on the way (§437.4); for every other variant, its
/// words with the numbers taken out.
fn keyed(why: &Error, which: WhichFuse) -> Refusal {
    let (variant, key) = if let Error::Undecidable { records } = why {
        let mut gave_up: Vec<_> = records
            .records()
            .iter()
            .filter(|record| matches!(record.outcome(), Decision::Undecidable { .. }))
            .map(|record| record.predicate())
            .collect();
        gave_up.sort_unstable();
        gave_up.dedup();
        let key = if gave_up.is_empty() {
            THE_SETTLED_DECISIONS_ADDED_UP_PAST_THE_BUDGET.to_owned()
        } else {
            gave_up.join(", ")
        };
        ("Undecidable".to_owned(), key)
    } else {
        (
            variant_named(&format!("{why:?}")),
            numbers_as_n(&on_one_line(&why.to_string())),
        )
    };
    Refusal {
        variant,
        key,
        which,
    }
}

/// `Invalid(DynamicRangeTooWide { .. })` as `Invalid(DynamicRangeTooWide)`:
/// the variant's name off the front of its `Debug`, and the name of the error
/// inside it where it wraps one.
fn variant_named(debug: &str) -> String {
    let name = |text: &str| -> String {
        text.chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect()
    };
    let outer = name(debug);
    let inner = debug[outer.len()..]
        .strip_prefix('(')
        .map(name)
        .unwrap_or_default();
    if inner.starts_with(|c: char| c.is_ascii_uppercase()) {
        format!("{outer}({inner})")
    } else {
        outer
    }
}

fn grouped(replayed: Vec<(PathBuf, Option<WhatOneFileDid>)>, time_limit: Duration) -> CorpusReplay {
    let mut replay = CorpusReplay {
        groups: Vec::new(),
        files: replayed.len(),
        files_that_refused: 0,
        panicked: Vec::new(),
        hung: Vec::new(),
        unreadable: Vec::new(),
        time_limit,
    };
    let mut groups: HashMap<(String, String), RefusalGroup> = HashMap::new();
    for (file, did) in replayed {
        let (refusals, panicked) = match did {
            None => {
                replay.hung.push(file);
                continue;
            }
            Some(WhatOneFileDid::Unreadable(why)) => {
                replay.unreadable.push((file, why));
                continue;
            }
            Some(WhatOneFileDid::Replayed { refusals, panicked }) => (refusals, panicked),
        };
        if !refusals.is_empty() {
            replay.files_that_refused += 1;
        }
        let mut met_here = HashSet::new();
        for refusal in refusals {
            let group = groups
                .entry((refusal.variant.clone(), refusal.key.clone()))
                .or_insert_with(|| RefusalGroup {
                    variant: refusal.variant.clone(),
                    key: refusal.key.clone(),
                    refusals: 0,
                    files: 0,
                    certified: 0,
                    tolerant: 0,
                    first_way_round: 0,
                    other_way_round: 0,
                    sample: file.clone(),
                });
            group.refusals += 1;
            if met_here.insert((refusal.variant, refusal.key)) {
                group.files += 1;
            }
            if refusal.which.policy == Policy::certified() {
                group.certified += 1;
            } else {
                group.tolerant += 1;
            }
            match refusal.which.way_round {
                WayRound::First => group.first_way_round += 1,
                WayRound::Other => group.other_way_round += 1,
            }
        }
        if let Some(said) = panicked {
            replay.panicked.push((file, said));
        }
    }
    replay.groups = groups.into_values().collect();
    replay.groups.sort_by(|a, b| {
        b.refusals
            .cmp(&a.refusals)
            .then_with(|| a.variant.cmp(&b.variant))
            .then_with(|| a.key.cmp(&b.key))
    });
    replay
}

/// A worker's stack. The fuzz target runs on libFuzzer's main thread, and a
/// stack overflow here would take the whole replay down rather than one file,
/// so this is set well above the 8 MiB of a main thread.
const STACK: usize = 64 << 20;

/// What a worker tells the thread keeping time.
enum Told<T> {
    Started {
        worker: usize,
        input: usize,
        at: Instant,
    },
    Finished {
        worker: usize,
        input: usize,
        said: T,
    },
}

/// `replay` run on every input, `workers` at a time, each answer in the order
/// the inputs came in. `None` is an input still running when `time_limit` ran
/// out: its worker is left with it, and another started in its place.
fn each_in_parallel<T: Send + 'static>(
    inputs: Vec<PathBuf>,
    workers: usize,
    time_limit: Duration,
    replay: fn(&Path) -> T,
) -> Vec<(PathBuf, Option<T>)> {
    let inputs = Arc::new(inputs);
    let next = Arc::new(AtomicUsize::new(0));
    let (tell, told) = mpsc::channel();
    let mut workers_started = 0;
    let mut start_a_worker = || {
        let (inputs, next, tell) = (Arc::clone(&inputs), Arc::clone(&next), tell.clone());
        let worker = workers_started;
        workers_started += 1;
        thread::Builder::new()
            .name(format!("replay-{worker}"))
            .stack_size(STACK)
            .spawn(move || {
                loop {
                    let input = next.fetch_add(1, Ordering::Relaxed);
                    let Some(path) = inputs.get(input) else {
                        return;
                    };
                    let at = Instant::now();
                    if tell.send(Told::Started { worker, input, at }).is_err() {
                        return;
                    }
                    let said = replay(path);
                    if tell
                        .send(Told::Finished {
                            worker,
                            input,
                            said,
                        })
                        .is_err()
                    {
                        return;
                    }
                }
            })
            .expect("a thread to replay on");
    };
    let total = inputs.len();
    for _ in 0..workers.clamp(1, total.max(1)) {
        start_a_worker();
    }

    let mut answers: Vec<Option<Option<T>>> = std::iter::repeat_with(|| None).take(total).collect();
    let mut known = 0;
    let mut running: HashMap<usize, (usize, Instant)> = HashMap::new();
    while known < total {
        let first_deadline = running.values().map(|&(_, at)| at + time_limit).min();
        let heard = match first_deadline {
            Some(deadline) => told.recv_timeout(deadline.saturating_duration_since(Instant::now())),
            None => told
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        match heard {
            Ok(Told::Started { worker, input, at }) => {
                running.insert(worker, (input, at));
            }
            Ok(Told::Finished {
                worker,
                input,
                said,
            }) => {
                if running.get(&worker).is_some_and(|&(on, _)| on == input) {
                    running.remove(&worker);
                }
                // A worker left behind can still finish; its input was already
                // counted as hung, and stays so.
                if answers[input].is_none() {
                    answers[input] = Some(Some(said));
                    known += 1;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let now = Instant::now();
                let late: Vec<usize> = running
                    .iter()
                    .filter(|&(_, &(_, at))| now >= at + time_limit)
                    .map(|(&worker, _)| worker)
                    .collect();
                for worker in late {
                    let (input, _) = running.remove(&worker).expect("a late worker was running");
                    answers[input] = Some(None);
                    known += 1;
                    start_a_worker();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                unreachable!("this thread holds a sender, so the channel stays open")
            }
        }
    }
    inputs
        .iter()
        .cloned()
        .zip(answers.into_iter().map(|answer| {
            answer.expect("every input either answered or ran out of time before the loop ended")
        }))
        .collect()
}
