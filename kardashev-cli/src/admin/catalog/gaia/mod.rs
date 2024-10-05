mod model;

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        atomic::AtomicUsize,
        Arc,
    },
};

use async_compression::tokio::bufread::GzipDecoder;
use color_eyre::eyre::{
    bail,
    Error,
};
use csv_async::{
    AsyncReaderBuilder,
    DeserializeRecordsIntoStream,
};
use futures::TryStreamExt;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use tokio::{
    fs::File,
    io::BufReader,
    sync::mpsc,
    task::JoinHandle,
};

pub use self::model::{
    astro::AstrophysicalParameters,
    source::GaiaSource,
};

lazy_static! {
    static ref FILE_NAME_REGEX: Regex = r"^(\w+)_(\d+)-(\d+).csv.gz$".parse().unwrap();
}

#[derive(Clone, Copy, Debug)]
pub struct HealPixRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug)]
pub struct Record {
    pub healpix_range: HealPixRange,
    pub gaia_source: GaiaSource,
    pub astrophysical_parameters: Option<AstrophysicalParameters>,
}

pub struct Data {
    partitions: Vec<Partition>,
}

impl Data {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        #[derive(Debug)]
        struct PartitionBuf {
            healpix_range: HealPixRange,
            gaia_source: Option<PathBuf>,
            astrophysical_parameters: Option<PathBuf>,
        }

        fn parse_file_name(file_name: &str) -> Option<(&str, HealPixRange)> {
            let captures = FILE_NAME_REGEX.captures(file_name)?;
            let prefix = captures.get(1)?.as_str();
            let start = captures.get(2)?.as_str().parse().ok()?;
            let end = captures.get(3)?.as_str().parse().ok()?;
            Some((prefix, HealPixRange { start, end }))
        }

        let mut read_dir = tokio::fs::read_dir(path).await?;

        let mut partitions = BTreeMap::new();

        while let Some(entry) = read_dir.next_entry().await? {
            let Ok(file_name) = entry.file_name().into_string()
            else {
                continue;
            };
            let Some((prefix, healpix_range)) = parse_file_name(&file_name)
            else {
                continue;
            };

            let partition = partitions.entry(healpix_range.start).or_insert_with(|| {
                PartitionBuf {
                    healpix_range,
                    gaia_source: None,
                    astrophysical_parameters: None,
                }
            });

            let path = entry.path();

            match prefix {
                "GaiaSource" => partition.gaia_source = Some(path),
                "AstrophysicalParameters" => partition.astrophysical_parameters = Some(path),
                _ => continue,
            }
        }

        let mut last_end = None;
        for (_, partition) in &partitions {
            if let Some(last_end) = last_end {
                if partition.healpix_range.start != last_end + 1 {
                    bail!(
                        "partition gap: last_end = {last_end}, next_start = {}",
                        partition.healpix_range.start
                    );
                }
            }
            last_end = Some(partition.healpix_range.end);
        }

        let partitions = partitions
            .into_iter()
            .filter_map(|(_, partition)| {
                match partition {
                    PartitionBuf {
                        healpix_range,
                        gaia_source: Some(gaia_source),
                        astrophysical_parameters: Some(astrophysical_parameters),
                    } => {
                        Some(Partition {
                            healpix_range,
                            gaia_source,
                            astrophysical_parameters,
                        })
                    }
                    PartitionBuf {
                        healpix_range,
                        gaia_source: None,
                        ..
                    } => {
                        tracing::warn!(?healpix_range, "GaiaSource missing");
                        None
                    }
                    PartitionBuf {
                        healpix_range,
                        astrophysical_parameters: None,
                        ..
                    } => {
                        tracing::warn!(?healpix_range, "AstrophysicalParameters missing");
                        None
                    }
                }
            })
            .collect();

        Ok(Self { partitions })
    }

    pub fn sequential<'a>(&'a self) -> SequentialReader<'a> {
        SequentialReader::new(self)
    }

    pub fn parallel(&self, num_threads: Option<usize>, buf_size: Option<usize>) -> ParallelReader {
        ParallelReader::new(self, num_threads, buf_size)
    }
}

#[derive(Clone, Debug)]
struct Partition {
    healpix_range: HealPixRange,
    gaia_source: PathBuf,
    astrophysical_parameters: PathBuf,
}

type Csv<T> = DeserializeRecordsIntoStream<'static, GzipDecoder<BufReader<File>>, T>;

struct PartitionReader {
    healpix_range: HealPixRange,
    gaia_source: Csv<GaiaSource>,
    astrophysical_parameters: Csv<AstrophysicalParameters>,
    astrophysical_parameters_buf: Option<AstrophysicalParameters>,
}

impl PartitionReader {
    pub async fn open(partition: &Partition) -> Result<Self, Error> {
        async fn open_reader<T: for<'de> Deserialize<'de> + 'static>(
            path: &Path,
        ) -> Result<Csv<T>, Error> {
            let file = File::open(path).await?;
            let reader = BufReader::new(file);
            let gzip_reader = GzipDecoder::new(reader);
            let stream = AsyncReaderBuilder::new()
                .comment(Some(b'#'))
                .delimiter(b',')
                .create_deserializer(gzip_reader)
                .into_deserialize();
            Ok(stream)
        }

        Ok(Self {
            healpix_range: partition.healpix_range,
            gaia_source: open_reader(&partition.gaia_source).await?,
            astrophysical_parameters: open_reader(&partition.astrophysical_parameters).await?,
            astrophysical_parameters_buf: None,
        })
    }

    pub async fn read_record(&mut self) -> Result<Option<Record>, Error> {
        loop {
            let Some(gaia_source) = self.gaia_source.try_next().await?
            else {
                return Ok(None);
            };

            if self.astrophysical_parameters_buf.is_none() {
                self.astrophysical_parameters_buf =
                    self.astrophysical_parameters.try_next().await?;
            }

            let astrophysical_parameters =
                if let Some(astrophysical_parameters) = &self.astrophysical_parameters_buf {
                    match gaia_source
                        .source_id
                        .cmp(&astrophysical_parameters.source_id)
                    {
                        Ordering::Equal => self.astrophysical_parameters_buf.take(),
                        Ordering::Less => None,
                        Ordering::Greater => {
                            // there should be an entry in GaiaSource for every record we find.
                            tracing::warn!(
                                source_id = astrophysical_parameters.source_id,
                                "missing GaiaSource"
                            );
                            self.astrophysical_parameters_buf = None;
                            None
                        }
                    }
                }
                else {
                    None
                };

            return Ok(Some(Record {
                healpix_range: self.healpix_range,
                gaia_source,
                astrophysical_parameters,
            }));
        }
    }
}

pub struct SequentialReader<'a> {
    partitions: &'a [Partition],
    index: usize,
    reader: Option<PartitionReader>,
}

impl<'a> SequentialReader<'a> {
    fn new(data: &'a Data) -> Self {
        Self {
            partitions: &data.partitions,
            index: 0,
            reader: None,
        }
    }

    fn next_partition(&mut self) -> Option<&Partition> {
        let partition = self.partitions.get(self.index)?;
        self.index += 1;
        Some(partition)
    }

    async fn reader(&mut self) -> Result<Option<&mut PartitionReader>, Error> {
        if self.reader.is_none() {
            let Some(partition) = self.next_partition()
            else {
                return Ok(None);
            };
            let reader = PartitionReader::open(partition).await?;
            self.reader = Some(reader);
        }

        let reader = self.reader.as_mut().unwrap();

        Ok(Some(reader))
    }

    pub async fn read_record(&mut self) -> Result<Option<Record>, Error> {
        let Some(reader) = self.reader().await?
        else {
            return Ok(None);
        };
        reader.read_record().await
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.index, self.partitions.len())
    }

    pub fn skip_file(&mut self) {
        self.reader = None;
    }
}

struct Queue {
    partitions: Vec<Partition>,
    index: AtomicUsize,
}

impl Queue {
    async fn next(&self) -> Option<&Partition> {
        let index = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.partitions.get(index)
    }

    pub fn progress(&self) -> (usize, usize) {
        let num_partitions = self.partitions.len();
        let index = self.index.load(std::sync::atomic::Ordering::SeqCst);
        let index = std::cmp::min(index, num_partitions);
        (index, num_partitions)
    }
}

#[derive(Clone)]
struct Work {
    queue: Arc<Queue>,
    records_tx: mpsc::Sender<Result<Record, Error>>,
}

impl Work {
    pub fn new(
        partitions: Vec<Partition>,
        buf_size: usize,
    ) -> (Self, mpsc::Receiver<Result<Record, Error>>) {
        let (records_tx, records_rx) = mpsc::channel(buf_size);

        let this = Self {
            queue: Arc::new(Queue {
                partitions,
                index: AtomicUsize::new(0),
            }),
            records_tx,
        };

        (this, records_rx)
    }

    pub fn spawn(&self) -> JoinHandle<()> {
        let this = self.clone();
        tokio::spawn(async move {
            if let Err(error) = this.run().await {
                let _ = this.records_tx.send(Err(error));
            }
        })
    }

    async fn run(&self) -> Result<(), Error> {
        'outer: while let Some(partition) = self.queue.next().await {
            let mut reader = PartitionReader::open(&partition).await?;

            while let Some(record) = reader.read_record().await? {
                if self.records_tx.send(Ok(record)).await.is_err() {
                    break 'outer;
                }
            }
        }

        Ok(())
    }

    pub fn progress(&self) -> (usize, usize) {
        self.queue.progress()
    }
}

pub struct ParallelReader {
    work: Work,
    join_handles: Vec<JoinHandle<()>>,
    records_rx: mpsc::Receiver<Result<Record, Error>>,
}

impl ParallelReader {
    fn new(data: &Data, parallel: Option<usize>, buf_size: Option<usize>) -> Self {
        let partitions = data.partitions.iter().cloned().collect();
        let buf_size = buf_size.unwrap_or(64);
        let (work, records_rx) = Work::new(partitions, buf_size);

        let parallel = parallel.unwrap_or_else(|| num_cpus::get());

        let mut join_handles = Vec::with_capacity(parallel);
        for _ in 0..parallel {
            join_handles.push(work.spawn());
        }

        Self {
            work,
            join_handles,
            records_rx,
        }
    }

    pub async fn read_record(&mut self) -> Result<Option<Record>, Error> {
        let record = self.records_rx.recv().await;

        if record.is_none() && !self.join_handles.is_empty() {
            // all senders dropped, join threads
            tracing::debug!("joining threads");
            for join_handle in self.join_handles.drain(..) {
                join_handle.await?;
            }
        }

        record.transpose()
    }

    pub fn progress(&self) -> (usize, usize) {
        self.work.progress()
    }
}
