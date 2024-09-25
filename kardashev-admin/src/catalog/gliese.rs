use std::{
    collections::{
        HashMap,
        HashSet,
    },
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
    pub ident: String,
    pub comp: Option<String>,
    pub dist_rel: Option<DistanceReliability>,
    pub ra: Option<RightAscension>,
    pub decl: Option<Declination>,
    pub pm: Option<ProperMotion>,
    pub rv: Option<f32>,
    pub n_rv: Option<RadialVelocityRemark>,
    pub sp: SpectralType,
    pub mag_v: Option<Magnitude>,
    pub mag_b: Option<Magnitude>,
    pub mag_u: Option<Magnitude>,
    pub mag_r: Option<Magnitude>,
    pub trplx: Option<Parallax>,
    pub plx: Option<Parallax>,
    pub n_plx: Option<ParallaxNote>,
    pub mv: f32,
    pub n_mv: Option<MagnitudeOrigin>,
    pub q_mv: AbsoluteMagnitudeQuality,
    pub velocities: Option<Velocities>,
    pub hd: String,
    pub dm: String,
    pub giclas: String,
    pub lhs: String,
    pub other: String,
    pub remarks: String,
}

/// s:  trig. parallax > 0.0390 and phot. parallax    <  0.0390
/// x:  trig. parallax > 0.0390 and phot. parallax    <  0.0190
/// p:  trig. parallax < 0.0390 and phot. parallax    >  0.0390
/// q:  trig. parallax < 0.0390 and phot. parallax(:) >  0.0390
#[derive(Clone, Copy, Debug)]
pub enum DistanceReliability {
    S,
    X,
    P,
    Q,
}

/// Right Ascension B1950
#[derive(Clone, Copy, Debug)]
pub struct RightAscension {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

/// Declination B1950
#[derive(Clone, Copy, Debug)]
pub struct Declination {
    pub sign: Sign,
    pub degrees: u16,
    pub minutes: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug)]
pub struct Magnitude {
    pub magnitude: f32,
    pub origin: Option<MagnitudeOrigin>,
    pub joint: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ProperMotion {
    pub total: f32,
    pub uncertain: bool,
    pub direction: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum RadialVelocityRemark {
    Variable,
    SuspectedSpectroscopicBinary,
    SpectroscopicBinary,
}

#[derive(Clone, Debug)]
pub struct SpectralType {
    pub ident: String,
    pub source: Option<Source>,
}

#[derive(Clone, Copy, Debug)]
pub enum Source {
    /// K    Kuiper Type (see ApJS, 59, 197, 1985)
    Kuiper,
    /// L    San-Gak Lee (AJ 89, 702, 1984)
    SanGakLee,
    /// O    objective prism MK type (but not Michigan type)
    ObjectivePrismMk,
    /// R    Robertson type (AJ, 89, 1229, 1984)
    Robertson,
    /// s    Stephenson type (AJ, 91, 144, 1985  and AJ, 92, 139, 1986)
    Stephenson,
    /// S    Smethells type (IAU Coll. No 76, p. 421, 1983)
    Smethells,
    /// U    Upgren et al. (AJ, 77, 486, 1972)
    Upgren,
    /// W    Mount Wilson type
    MountWilson,
    Other(char),
}

/// P    photographic
/// * photometric
/// C    from 'Cape refractor system'
/// c    calculated or transformed
/// v    variable
/// :    uncertain
#[derive(Clone, Copy, Debug)]
pub enum MagnitudeOrigin {
    Photographic,
    Photometric,
    CapeRefractorSystem,
    CalculatedOrTransformed,
    Variable,
    Uncertain,
    Other(char),
}

impl MagnitudeOrigin {
    fn parse(s: &str) -> Result<Option<Self>, Error> {
        Ok(match s {
            "" => None,
            "P" => Some(Self::Photographic),
            "*" => Some(Self::Photometric),
            "C" => Some(Self::CapeRefractorSystem),
            "c" => Some(Self::CalculatedOrTransformed),
            "v" => Some(Self::Variable),
            ":" => Some(Self::Uncertain),
            _ => Some(Self::Other(s.chars().next().unwrap())),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Parallax {
    pub parallax: f32,
    pub error: f32,
}

/// r    parallax from spectral types and broad-band colors
/// w    photom. parallax for white dwarfs
/// s    photom. parallax from Stroemgren photometry
/// o    photom. parallax from Stroemgren photometry
///      calculated by E. H. Olsen
/// p    photom. parallax from other colors
#[derive(Clone, Copy, Debug)]
pub enum ParallaxNote {
    R,
    W,
    S,
    O,
    P,
    Other(char),
}

/// a               s.e.  <  0.10 mag
/// b    0.11  <    s.e.  <  0.20
/// c    0.21  <    s.e.  <  0.30
/// d    0.31  <    s.e.  <  0.50
/// e    0.51  <    s.e.  <  0.75
/// f    0.76  <    s.e.
#[derive(Clone, Copy, Debug)]
pub enum AbsoluteMagnitudeQuality {
    A,
    B,
    C,
    D,
    E,
    F,
}

#[derive(Clone, Copy, Debug)]
pub struct Velocities {
    /// space velocity component in the galactic plane and directed to the
    /// galactic center
    pub u: i16,
    /// space velocity component in the galactic plane and in the direction of
    /// galactic rotation
    pub v: i16,
    /// space velocity component in the galactic plane and in the direction of
    /// the North Galactic Pole
    pub w: i16,
}

pub struct Reader {
    reader: BufReader<File>,
    line_buf: String,
}

impl Reader {
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

        let ident = part(0..8);
        let comp = part(8..10);
        let comp = if comp.is_empty() { None } else { Some(comp) };

        let dist_rel = match part(10..11) {
            "" => None,
            "s" => Some(DistanceReliability::S),
            "x" => Some(DistanceReliability::X),
            "p" => Some(DistanceReliability::P),
            "q" => Some(DistanceReliability::Q),
            s => bail!("Invalid distance reliability: {s}"),
        };
        let ra = {
            let hours = part(12..14);
            let minutes = part(15..17);
            let seconds = part(18..20);

            if hours.is_empty() && minutes.is_empty() && seconds.is_empty() {
                None
            }
            else {
                Some(RightAscension {
                    hours: hours.parse()?,
                    minutes: minutes.parse()?,
                    seconds: seconds.parse()?,
                })
            }
        };

        let decl = {
            let sign = part(21..22);
            let degrees = part(22..24);
            let minutes = part(25..29);

            if sign.is_empty() && degrees.is_empty() && minutes.is_empty() {
                None
            }
            else {
                Some(Declination {
                    sign: match sign {
                        "+" => Sign::Positive,
                        "-" => Sign::Negative,
                        s => bail!("Invalid sign: {s}"),
                    },
                    degrees: degrees.parse()?,
                    minutes: minutes.parse()?,
                })
            }
        };

        let pm = {
            let total = part(30..36);
            let uncertain = part(36..37);
            let direction = part(37..42);

            if total.is_empty() && uncertain.is_empty() && direction.is_empty() {
                None
            }
            else {
                Some(ProperMotion {
                    total: total.parse()?,
                    uncertain: uncertain == ":",
                    direction: direction.parse()?,
                })
            }
        };

        let rv = part(42..49);
        let rv = if rv.is_empty() {
            None
        }
        else {
            Some(rv.parse()?)
        };
        let n_rv = match part(50..53) {
            "" => None,
            "VAR" => Some(RadialVelocityRemark::Variable),
            "SB?" => Some(RadialVelocityRemark::SuspectedSpectroscopicBinary),
            "SB" => Some(RadialVelocityRemark::SpectroscopicBinary),
            s => bail!("Invalid radial velocity remark: {s}"),
        };

        let sp = {
            let ident = part(54..66);
            let source = match part(66..67) {
                "" => None,
                "K" => Some(Source::Kuiper),
                "L" => Some(Source::SanGakLee),
                "O" => Some(Source::ObjectivePrismMk),
                "R" => Some(Source::Robertson),
                "s" => Some(Source::Stephenson),
                "S" => Some(Source::Smethells),
                "U" => Some(Source::Upgren),
                "W" => Some(Source::MountWilson),
                s => Some(Source::Other(s.chars().next().unwrap())),
            };
            SpectralType {
                ident: ident.to_owned(),
                source,
            }
        };

        let parse_magnitude = |r_mag: Range<usize>| -> Result<Option<Magnitude>, Error> {
            let magnitude = part(r_mag.clone());
            let r_origin = r_mag.end..r_mag.end + 1;
            let origin = part(r_origin);
            let r_joint = r_mag.end + 1..r_mag.end + 2;
            let joint = part(r_joint);
            if magnitude.is_empty() && origin.is_empty() && joint.is_empty() {
                Ok(None)
            }
            else {
                Ok(Some(Magnitude {
                    magnitude: magnitude.parse()?,
                    origin: MagnitudeOrigin::parse(origin)?,
                    joint: joint == "J",
                }))
            }
        };

        let mag_v = parse_magnitude(67..73)?;
        let mag_b = parse_magnitude(75..80)?;
        let mag_u = parse_magnitude(82..87)?;
        let mag_r = parse_magnitude(89..94)?;

        let trplx = {
            let trplx = part(96..102);
            let e_trplx = part(102..107);
            if trplx.is_empty() && e_trplx.is_empty() {
                None
            }
            else {
                Some(Parallax {
                    parallax: trplx.parse()?,
                    error: e_trplx.parse()?,
                })
            }
        };

        let plx = {
            let plx = part(108..114);
            let e_plx = part(114..119);
            if plx.is_empty() && e_plx.is_empty() {
                None
            }
            else {
                Some(Parallax {
                    parallax: plx.parse()?,
                    error: e_plx.parse()?,
                })
            }
        };

        let n_plx = match part(119..120) {
            "" => None,
            "r" => Some(ParallaxNote::R),
            "w" => Some(ParallaxNote::W),
            "s" => Some(ParallaxNote::S),
            "o" => Some(ParallaxNote::O),
            "p" => Some(ParallaxNote::P),
            s => Some(ParallaxNote::Other(s.chars().next().unwrap())),
        };

        let mv = part(121..126).parse::<f32>()?;
        let n_mv = MagnitudeOrigin::parse(part(126..128))?;
        let q_mv = match part(128..129) {
            "a" => AbsoluteMagnitudeQuality::A,
            "b" => AbsoluteMagnitudeQuality::B,
            "c" => AbsoluteMagnitudeQuality::C,
            "d" => AbsoluteMagnitudeQuality::D,
            "e" => AbsoluteMagnitudeQuality::E,
            "f" => AbsoluteMagnitudeQuality::F,
            s => bail!("Invalid absolute magnitude quality: {s}"),
        };

        let velocities = {
            let vel_u = part(131..135);
            let vel_v = part(136..140);
            let vel_w = part(141..145);
            if vel_u.is_empty() && vel_v.is_empty() && vel_w.is_empty() {
                None
            }
            else {
                Some(Velocities {
                    u: vel_u.parse()?,
                    v: vel_v.parse()?,
                    w: vel_w.parse()?,
                })
            }
        };

        let hd = part(146..152);
        let dm = part(153..165);
        let giclas = part(166..175);
        let lhs = part(176..181);
        let other = part(182..187);
        let remarks = part(188..257);

        Ok(Some(Record {
            ident: ident.to_owned(),
            comp: comp.map(ToOwned::to_owned),
            dist_rel,
            ra,
            decl,
            pm,
            rv,
            n_rv,
            sp,
            mag_v,
            mag_b,
            mag_u,
            mag_r,
            trplx,
            plx,
            n_plx,
            mv,
            n_mv,
            q_mv,
            velocities,
            hd: hd.to_owned(),
            dm: dm.to_owned(),
            giclas: giclas.to_owned(),
            lhs: lhs.to_owned(),
            other: other.to_owned(),
            remarks: remarks.to_owned(),
        }))
    }
}

pub fn get_names(record: &Record) -> Vec<String> {
    let greek = [
        ("ALP", "Alpha"),
        ("BET", "Beta"),
        ("GAM", "Gamma"),
        ("DEL", "Delta"),
        ("EPS", "Epsilon"),
        ("ZET", "Zeta"),
        ("ETA", "Eta"),
        ("THE", "Theta"),
        ("IOT", "Iota"),
        ("KAP", "Kapppa"),
        ("LAM", "Lambda"),
        ("MU", "Mu"),
        ("NU", "Nu"),
        ("XI", "Xi"),
        ("OMI", "Omicron"),
        ("PI", "Pi"),
        ("RHO", "Rho"),
        ("SIG", "Sigma"),
        ("TAU", "Tau"),
        ("UPS", "Upsilon"),
        ("PHI", "Phi"),
        ("CHI", "Chi"),
        ("PSI", "Psi"),
        ("OME", "Omega"),
    ]
    .into_iter()
    .collect::<HashMap<&'static str, &'static str>>();
    let constellations = [
        ("And", "Andromeda"),
        ("Ant", "Antlia"),
        ("Aps", "Apus"),
        ("Aqr", "Aquarius"),
        ("Aql", "Aquila"),
        ("Ara", "Ara"),
        ("Ari", "Aries"),
        ("Aur", "Auriga"),
        ("Boo", "Bootes"),
        ("Cae", "Caelum"),
        ("Cam", "Camelopardalis"),
        ("Cnc", "Cancer"),
        ("CVn", "Canes Venatici"),
        ("CMa", "Canis Major"),
        ("CMi", "Canis Minor"),
        ("Cap", "Capricornus"),
        ("Car", "Carina"),
        ("Cas", "Cassiopeia"),
        ("Cen", "Centaurus"),
        ("Cep", "Cepheus"),
        ("Cet", "Cetus"),
        ("Cha", "Chamaeleon"),
        ("Cir", "Circinus"),
        ("Col", "Columba"),
        ("Com", "Coma Berenices"),
        ("CrA", "Corona Australis"),
        ("CrB", "Corona Borealis"),
        ("Crv", "Corvus"),
        ("Crt", "Crater"),
        ("Cru", "Crux"),
        ("Cyg", "Cygnus"),
        ("Del", "Delphinus"),
        ("Dor", "Dorado"),
        ("Dra", "Draco"),
        ("Equ", "Equuleus"),
        ("Eri", "Eridanus"),
        ("For", "Fornax"),
        ("Gem", "Gemini"),
        ("Gru", "Grus"),
        ("Her", "Hercules"),
        ("Hor", "Horologium"),
        ("Hya", "Hydra"),
        ("Hyi", "Hydrus"),
        ("Ind", "Indus"),
        ("Lac", "Lacerta"),
        ("Leo", "Leo"),
        ("LMi", "Leo Minor"),
        ("Lep", "Lepus"),
        ("Lib", "Libra"),
        ("Lup", "Lupus"),
        ("Lyn", "Lynx"),
        ("Lyr", "Lyra"),
        ("Men", "Mensa"),
        ("Mic", "Microscopium"),
        ("Mon", "Monoceros"),
        ("Mus", "Musca"),
        ("Nor", "Norma"),
        ("Oct", "Octans"),
        ("Oph", "Ophiuchus"),
        ("Ori", "Orion"),
        ("Pav", "Pavo"),
        ("Peg", "Pegasus"),
        ("Per", "Perseus"),
        ("Phe", "Phoenix"),
        ("Pic", "Pictor"),
        ("Psc", "Pisces"),
        ("PsA", "Piscis Austrinius"),
        ("Pup", "Puppis"),
        ("Pyx", "Pyxis"),
        ("Ret", "Reticulum"),
        ("Sge", "Sagitta"),
        ("Sgr", "Sagittarius"),
        ("Sco", "Scorpius"),
        ("Scl", "Sculptor"),
        ("Sct", "Scutum"),
        ("Ser", "Serpens"),
        ("Sex", "Sextans"),
        ("Tau", "Taurus"),
        ("Tel", "Telescopium"),
        ("Tri", "Triangulum"),
        ("TrA", "Triangulum Australe"),
        ("Tuc", "Tucana"),
        ("UMa", "Ursa Major"),
        ("UMi", "Ursa Minor"),
        ("Vel", "Vela"),
        ("Vir", "Virgo"),
        ("Vol", "Volans"),
        ("Vul", "Vulpecula"),
    ]
    .into_iter()
    .collect::<HashMap<&'static str, &'static str>>();
    let generic = ["Wolf", "Ross"]
        .into_iter()
        .collect::<HashSet<&'static str>>();

    let mut names = vec![];
    let mut words = record.remarks.split_ascii_whitespace();
    while let Some(word) = words.next() {
        if let Some(word) = greek.get(word) {
            if let Some(word2) = words.next() {
                if let Some(word2) = constellations.get(word2) {
                    names.push(format!("{word} {word2}"))
                }
            }
        }

        if generic.contains(word) {
            let mut name = word.to_owned();
            if let Some(word) = words.next() {
                name.push(' ');
                name.push_str(word);
            }
            names.push(name);
        }
    }

    names
}
