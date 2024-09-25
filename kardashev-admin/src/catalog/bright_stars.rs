use std::{
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
    ops::Range,
    path::Path,
};

use color_eyre::eyre::{
    bail,
    Error,
};

#[derive(Clone, Debug)]
pub struct Record {
    pub hr: u16,
    pub name: Option<String>,
    pub dm: String,
    pub hd: Option<u32>,
    pub sao: Option<u32>,
    pub fk5: Option<u16>,
    pub pos_j2000: Option<Position>,
    pub spectral_type: String,
    pub parallax: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub right_ascension: RightAscension,
    pub declination: Declination,
}

#[derive(Clone, Copy, Debug)]
pub struct RightAscension {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Declination {
    pub sign: Sign,
    pub degrees: u8,
    pub minutes: u8,
    pub seconds: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct GalacticCoordinates {
    pub longitude: f32,
    pub latitude: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

pub struct CatalogReader {
    reader: BufReader<File>,
    line_buf: String,
}

impl CatalogReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
            line_buf: String::new(),
        })
    }

    pub fn read_record(&mut self) -> Result<Option<Record>, Error> {
        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        if self.line_buf.is_empty() {
            return Ok(None);
        }

        let part = |range: Range<usize>| {
            let start = range.start.min(self.line_buf.len());
            let end = range.end.min(self.line_buf.len());
            self.line_buf[start..end].trim_ascii()
        };
        let part_opt = |range: Range<usize>| {
            let s = part(range);
            if s.is_empty() {
                None
            }
            else {
                Some(s)
            }
        };

        let hr = part(0..4).parse()?;
        let name = part_opt(4..14).map(ToOwned::to_owned);
        let dm = part(14..25).to_owned();
        let hd = part_opt(37..41).map(|s| s.parse()).transpose()?;
        let sao = part_opt(37..41).map(|s| s.parse()).transpose()?;
        let fk5 = part_opt(37..41).map(|s| s.parse()).transpose()?;

        let pos_j2000 = {
            /*
            76- 77  I2     h       RAh      ?Hours RA, equinox J2000, epoch 2000.0 (1)
            78- 79  I2     min     RAm      ?Minutes RA, equinox J2000, epoch 2000.0 (1)
            80- 83  F4.1   s       RAs      ?Seconds RA, equinox J2000, epoch 2000.0 (1)
                84  A1     ---     DE-      ?Sign Dec, equinox J2000, epoch 2000.0 (1)
            85- 86  I2     deg     DEd      ?Degrees Dec, equinox J2000, epoch 2000.0 (1)
            87- 88  I2     arcmin  DEm      ?Minutes Dec, equinox J2000, epoch 2000.0 (1)
            89- 90  I2     arcsec  DEs      ?Seconds Dec, equinox J2000, epoch 2000.0 (1)
                      */
            if part(75..90).is_empty() {
                None
            }
            else {
                let ra_hours = part(75..77).parse()?;
                let ra_min = part(77..79).parse()?;
                let ra_sec = part(79..83).parse()?;
                let de_sign = match part(83..84) {
                    "+" => Sign::Positive,
                    "-" => Sign::Negative,
                    s => bail!("Invalid sign: {s}"),
                };
                let de_deg = part(84..86).parse()?;
                let de_min = part(86..88).parse()?;
                let de_sec = part(89..90).parse()?;

                Some(Position {
                    right_ascension: RightAscension {
                        hours: ra_hours,
                        minutes: ra_min,
                        seconds: ra_sec,
                    },
                    declination: Declination {
                        sign: de_sign,
                        degrees: de_deg,
                        minutes: de_min,
                        seconds: de_sec,
                    },
                })
            }
        };

        //  128-147  A20    ---     SpType   Spectral type
        let spectral_type = part(127..147).to_owned();

        // 162-166  F5.3   arcsec  Parallax ? Trigonometric parallax (unless n_Parallax)
        let parallax = part_opt(161..166).map(|s| s.parse()).transpose()?;

        Ok(Some(Record {
            hr,
            name,
            dm,
            hd,
            sao,
            fk5,
            pos_j2000,
            spectral_type,
            parallax,
        }))
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub hr: u16,
    pub count: u8,
    pub category: NoteCategory,
    pub remark: String,
}

#[derive(Clone, Copy, Debug)]
pub enum NoteCategory {
    Colors,
    Multiple,
    DynamicalParallaxes,
    GroupMembership,
    Miscellaneous,
    StarNames,
    Polarization,
    StellarRadii,
    Velocities,
    Spectra,
    SpectroscopicBinaries,
    Variability,
}

pub struct NotesReader {
    reader: BufReader<File>,
    line_buf: String,
}

impl NotesReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
            line_buf: String::new(),
        })
    }

    pub fn read_note(&mut self) -> Result<Option<Note>, Error> {
        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        if self.line_buf.is_empty() {
            return Ok(None);
        }

        let part = |range: Range<usize>| {
            let start = range.start.min(self.line_buf.len());
            let end = range.end.min(self.line_buf.len());
            self.line_buf[start..end].trim_ascii()
        };

        /*
        --------------------------------------------------------------------------------
           Bytes Format  Units  Label     Explanations
        --------------------------------------------------------------------------------
           2-  5  I4     ---    HR        [1/9110]+= Harvard Revised (HR)
           6-  7  I2     ---    Count     Note counter (sequential for a star)
           8- 11  A4     ---    Category *[A-Z: ] Remark category abbreviation:
          13-132  A120   ---    Remark    Remarks in free form text
                 */

        let hr = part(1..5).parse()?;
        let count = part(5..7).parse()?;
        let category = match part(7..11) {
            /*
            C   - Colors;
            D   - Double and multiple stars;
            DYN - Dynamical parallaxes;
            G   - Group membership;
            M   - Miscellaneous.
            N   - Star names;
            P   - Polarization;
            R   - Stellar radii or diameters;
            RV  - Radial and/or rotational velocities;
            S   - Spectra;
            SB  - Spectroscopic binaries;
            VAR - Variability;
            The category abbreviation is always followed by a colon (:).
                     */
            "C:" => NoteCategory::Colors,
            "D:" => NoteCategory::Multiple,
            "DYN:" => NoteCategory::DynamicalParallaxes,
            "G:" => NoteCategory::GroupMembership,
            "M:" => NoteCategory::Miscellaneous,
            "N:" => NoteCategory::StarNames,
            "P:" => NoteCategory::Polarization,
            "R:" => NoteCategory::StellarRadii,
            "RV:" => NoteCategory::Velocities,
            "S:" => NoteCategory::Spectra,
            "SB:" => NoteCategory::SpectroscopicBinaries,
            "VAR:" | "VAR" => NoteCategory::Variability,
            s => bail!("Invalid note category: {s}"),
        };
        let remark = part(12..132).to_owned();

        Ok(Some(Note {
            hr,
            count,
            category,
            remark,
        }))
    }
}
