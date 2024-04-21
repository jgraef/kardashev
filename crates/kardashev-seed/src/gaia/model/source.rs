use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::invalid_option;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaiaSource {
    /// Solution Identifier
    pub solution_id: u64,

    /// Unique source designation (unique across all Data Releases)
    pub designation: String,

    /// Unique source identifier (unique within a particular Data Release)
    pub source_id: u64,

    /// Random index for use when selecting subsets
    #[serde(deserialize_with = "invalid_option")]
    pub random_index: Option<i64>,

    /// Reference epoch
    ///
    /// Unit: yr
    #[serde(deserialize_with = "invalid_option")]
    pub ref_epoch: Option<f64>,

    /// Right ascension
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub ra: Option<f64>,

    /// Standard error of right ascension
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub ra_error: Option<f32>,

    /// Declination
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub dec: Option<f64>,

    /// Standard error of declination
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub dec_error: Option<f32>,

    /// Parallax
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub parallax: Option<f64>,

    /// Standard error of parallax
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub parallax_error: Option<f32>,

    /// Parallax divided by its standard error
    #[serde(deserialize_with = "invalid_option")]
    pub parallax_over_error: Option<f32>,

    /// Total proper motion
    ///
    /// Unit: mas.yr**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pm: Option<f32>,

    /// Proper motion in right ascension direction
    ///
    /// Unit: mas.yr**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pmra: Option<f64>,

    /// Standard error of proper motion in right ascension direction
    ///
    /// Unit: mas.yr**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pmra_error: Option<f32>,

    /// Proper motion in declination direction
    ///
    /// Unit: mas.yr**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pmdec: Option<f64>,

    /// Standard error of proper motion in declination direction
    ///
    /// Unit: mas.yr**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pmdec_error: Option<f32>,

    /// Correlation between right ascension and declination
    #[serde(deserialize_with = "invalid_option")]
    pub ra_dec_corr: Option<f32>,

    /// Correlation between right ascension and parallax
    #[serde(deserialize_with = "invalid_option")]
    pub ra_parallax_corr: Option<f32>,

    /// Correlation between right ascension and proper motion in right ascension
    #[serde(deserialize_with = "invalid_option")]
    pub ra_pmra_corr: Option<f32>,

    /// Correlation between right ascension and proper motion in declination
    #[serde(deserialize_with = "invalid_option")]
    pub ra_pmdec_corr: Option<f32>,

    /// Correlation between declination and parallax
    #[serde(deserialize_with = "invalid_option")]
    pub dec_parallax_corr: Option<f32>,

    /// Correlation between declination and proper motion in right ascension
    #[serde(deserialize_with = "invalid_option")]
    pub dec_pmra_corr: Option<f32>,

    /// Correlation between declination and proper motion in declination
    #[serde(deserialize_with = "invalid_option")]
    pub dec_pmdec_corr: Option<f32>,

    /// Correlation between parallax and proper motion in right ascension
    #[serde(deserialize_with = "invalid_option")]
    pub parallax_pmra_corr: Option<f32>,

    /// Correlation between parallax and proper motion in declination
    #[serde(deserialize_with = "invalid_option")]
    pub parallax_pmdec_corr: Option<f32>,

    /// Correlation between proper motion in right ascension and proper motion
    /// in declination
    #[serde(deserialize_with = "invalid_option")]
    pub pmra_pmdec_corr: Option<f32>,

    /// Total number of observations in the along-scan (AL) direction
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_n_obs_al: Option<i16>,

    /// Total number of observations in the across-scan (AC) direction
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_n_obs_ac: Option<i16>,

    /// Number of good observations in the along-scan (AL) direction
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_n_good_obs_al: Option<i16>,

    /// Number of bad observations in the along-scan (AL) direction
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_n_bad_obs_al: Option<i16>,

    /// Goodness of fit statistic of model wrt along-scan observations
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_gof_al: Option<f32>,

    /// AL chi-square value
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_chi2_al: Option<f32>,

    /// Excess noise of the source
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_excess_noise: Option<f32>,

    /// Significance of excess noise
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_excess_noise_sig: Option<f32>,

    /// Which parameters have been solved for?
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_params_solved: Option<i8>,

    /// Primary or seconday
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_primary_flag: Option<bool>,

    /// Effective wavenumber of the source used in the astrometric solution
    ///
    /// Unit: um**-1
    #[serde(deserialize_with = "invalid_option")]
    pub nu_eff_used_in_astrometry: Option<f32>,

    /// Astrometrically estimated pseudocolour of the source
    ///
    /// Unit: um**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pseudocolour: Option<f32>,

    /// Standard error of the pseudocolour of the source
    ///
    /// Unit: um**-1
    #[serde(deserialize_with = "invalid_option")]
    pub pseudocolour_error: Option<f32>,

    /// Correlation between right ascension and pseudocolour
    #[serde(deserialize_with = "invalid_option")]
    pub ra_pseudocolour_corr: Option<f32>,

    /// Correlation between declination and pseudocolour
    #[serde(deserialize_with = "invalid_option")]
    pub dec_pseudocolour_corr: Option<f32>,

    /// Correlation between parallax and pseudocolour
    #[serde(deserialize_with = "invalid_option")]
    pub parallax_pseudocolour_corr: Option<f32>,

    /// Correlation between proper motion in right asension and pseudocolour
    #[serde(deserialize_with = "invalid_option")]
    pub pmra_pseudocolour_corr: Option<f32>,

    /// Correlation between proper motion in declination and pseudocolour
    #[serde(deserialize_with = "invalid_option")]
    pub pmdec_pseudocolour_corr: Option<f32>,

    /// Matched FOV transits used in the AGIS solution
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_matched_transits: Option<i16>,

    /// Number of visibility periods used in Astrometric solution
    #[serde(deserialize_with = "invalid_option")]
    pub visibility_periods_used: Option<i16>,

    /// The longest semi-major axis of the 5-d error ellipsoid
    ///
    /// Unit: mas
    #[serde(deserialize_with = "invalid_option")]
    pub astrometric_sigma5d_max: Option<f32>,

    /// The number of transits matched to this source
    #[serde(deserialize_with = "invalid_option")]
    pub matched_transits: Option<i16>,

    /// The number of transits newly incorporated into an existing source in the
    /// current cycle
    #[serde(deserialize_with = "invalid_option")]
    pub new_matched_transits: Option<i16>,

    /// The number of transits removed from an existing source in the current
    /// cycle
    #[serde(deserialize_with = "invalid_option")]
    pub matched_transits_removed: Option<i16>,

    /// Amplitude of the IPD GoF versus position angle of scan
    #[serde(deserialize_with = "invalid_option")]
    pub ipd_gof_harmonic_amplitude: Option<f32>,

    /// Phase of the IPD GoF versus position angle of scan
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub ipd_gof_harmonic_phase: Option<f32>,

    /// Percent of successful-IPD windows with more than one peak
    #[serde(deserialize_with = "invalid_option")]
    pub ipd_frac_multi_peak: Option<i8>,

    /// Percent of transits with truncated windows or multiple gate
    #[serde(deserialize_with = "invalid_option")]
    pub ipd_frac_odd_win: Option<i8>,

    /// Renormalised unit weight error
    #[serde(deserialize_with = "invalid_option")]
    pub ruwe: Option<f32>,

    /// Degree of concentration of scan directions across the source
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_strength_k1: Option<f32>,

    /// Degree of concentration of scan directions across the source
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_strength_k2: Option<f32>,

    /// Degree of concentration of scan directions across the source
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_strength_k3: Option<f32>,

    /// Degree of concentration of scan directions across the source
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_strength_k4: Option<f32>,

    /// Mean position angle of scan directions across the source
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_mean_k1: Option<f32>,

    /// Mean position angle of scan directions across the source
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_mean_k2: Option<f32>,

    /// Mean position angle of scan directions across the source
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_mean_k3: Option<f32>,

    /// Mean position angle of scan directions across the source
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub scan_direction_mean_k4: Option<f32>,

    /// Source with multiple source identifiers
    #[serde(deserialize_with = "invalid_option")]
    pub duplicated_source: Option<bool>,

    /// Number of observations contributing to G photometry
    #[serde(deserialize_with = "invalid_option")]
    pub phot_g_n_obs: Option<i16>,

    /// G-band mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_g_mean_flux: Option<f64>,

    /// Error on G-band mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_g_mean_flux_error: Option<f32>,

    /// G-band mean flux divided by its error
    #[serde(deserialize_with = "invalid_option")]
    pub phot_g_mean_flux_over_error: Option<f32>,

    /// G-band mean magnitude
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub phot_g_mean_mag: Option<f32>,

    /// Number of observations contributing to BP photometry
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_n_obs: Option<i16>,

    /// Integrated BP mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_mean_flux: Option<f64>,

    /// Error on the integrated BP mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_mean_flux_error: Option<f32>,

    /// Integrated BP mean flux divided by its error
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_mean_flux_over_error: Option<f32>,

    /// Integrated BP mean magnitude
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_mean_mag: Option<f32>,

    /// Number of observations contributing to RP photometry
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_n_obs: Option<i16>,

    /// Integrated RP mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_mean_flux: Option<f64>,

    /// Error on the integrated RP mean flux
    ///
    /// Unit: 'electron'.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_mean_flux_error: Option<f32>,

    /// Integrated RP mean flux divided by its error
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_mean_flux_over_error: Option<f32>,

    /// Integrated RP mean magnitude
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_mean_mag: Option<f32>,

    /// BP/RP excess factor
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_rp_excess_factor: Option<f32>,

    /// Number of BP contaminated transits
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_n_contaminated_transits: Option<i16>,

    /// Number of BP blended transits
    #[serde(deserialize_with = "invalid_option")]
    pub phot_bp_n_blended_transits: Option<i16>,

    /// Number of RP contaminated transits
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_n_contaminated_transits: Option<i16>,

    /// Number of RP blended transits
    #[serde(deserialize_with = "invalid_option")]
    pub phot_rp_n_blended_transits: Option<i16>,

    /// Photometry processing mode
    #[serde(deserialize_with = "invalid_option")]
    pub phot_proc_mode: Option<i8>,

    /// BP - RP colour
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub bp_rp: Option<f32>,

    /// BP - G colour
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub bp_g: Option<f32>,

    /// G - RP colour
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub g_rp: Option<f32>,

    /// Radial velocity
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub radial_velocity: Option<f32>,

    /// Radial velocity error
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub radial_velocity_error: Option<f32>,

    /// Method used to obtain the radial velocity
    #[serde(deserialize_with = "invalid_option")]
    pub rv_method_used: Option<i8>,

    /// Number of transits used to compute the radial velocity
    #[serde(deserialize_with = "invalid_option")]
    pub rv_nb_transits: Option<i16>,

    /// Number of valid transits that have undergone deblending
    #[serde(deserialize_with = "invalid_option")]
    pub rv_nb_deblended_transits: Option<i16>,

    /// Number of visibility periods used to estimate the radial velocity
    #[serde(deserialize_with = "invalid_option")]
    pub rv_visibility_periods_used: Option<i16>,

    /// Expected signal to noise ratio in the combination of the spectra used to
    /// obtain the radial velocity
    #[serde(deserialize_with = "invalid_option")]
    pub rv_expected_sig_to_noise: Option<f32>,

    /// Radial velocity renormalised goodness of fit
    #[serde(deserialize_with = "invalid_option")]
    pub rv_renormalised_gof: Option<f32>,

    /// P-value for constancy based on a chi-squared criterion
    #[serde(deserialize_with = "invalid_option")]
    pub rv_chisq_pvalue: Option<f32>,

    /// Time coverage of the radial velocity time series
    ///
    /// Unit: d
    #[serde(deserialize_with = "invalid_option")]
    pub rv_time_duration: Option<f32>,

    /// Total amplitude in the radial velocity time series after outlier removal
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub rv_amplitude_robust: Option<f32>,

    /// Teff of the template used to compute the radial velocity
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub rv_template_teff: Option<f32>,

    /// Logg of the template used to compute the radial velocity
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub rv_template_logg: Option<f32>,

    /// [Fe/H] of the template used to compute the radial velocityy
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub rv_template_fe_h: Option<f32>,

    /// Origin of the atmospheric parameters associated to the template
    #[serde(deserialize_with = "invalid_option")]
    pub rv_atm_param_origin: Option<i16>,

    /// Spectral line broadening parameter
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub vbroad: Option<f32>,

    /// Uncertainty on the spectral line broadening
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub vbroad_error: Option<f32>,

    /// Number of transits used to compute vbroad
    #[serde(deserialize_with = "invalid_option")]
    pub vbroad_nb_transits: Option<i16>,

    /// Integrated Grvs magnitude
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub grvs_mag: Option<f32>,

    /// Grvs magnitude uncertainty
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub grvs_mag_error: Option<f32>,

    /// Number of transits used to compute Grvs
    #[serde(deserialize_with = "invalid_option")]
    pub grvs_mag_nb_transits: Option<i16>,

    /// Signal to noise ratio in the mean RVS spectrum
    #[serde(deserialize_with = "invalid_option")]
    pub rvs_spec_sig_to_noise: Option<f32>,

    /// Photometric variability flag
    #[serde(deserialize_with = "invalid_option")]
    pub phot_variable_flag: Option<String>,

    /// Galactic longitude
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub l: Option<f64>,

    /// Galactic latitude
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub b: Option<f64>,

    /// Ecliptic longitude
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub ecl_lon: Option<f64>,

    /// Ecliptic latitude
    ///
    /// Unit: deg
    #[serde(deserialize_with = "invalid_option")]
    pub ecl_lat: Option<f64>,

    /// Flag indicating the availability of additional information in the QSO
    /// candidates table
    #[serde(deserialize_with = "invalid_option")]
    pub in_qso_candidates: Option<bool>,

    /// Flag indicating the availability of additional information in the galaxy
    /// candidates table
    #[serde(deserialize_with = "invalid_option")]
    pub in_galaxy_candidates: Option<bool>,

    /// Flag indicating the availability of additional information in the
    /// various Non-Single Star tables
    #[serde(deserialize_with = "invalid_option")]
    pub non_single_star: Option<i16>,

    /// Flag indicating the availability of mean BP/RP spectrum in continuous
    /// representation for this source
    #[serde(deserialize_with = "invalid_option")]
    pub has_xp_continuous: Option<bool>,

    /// Flag indicating the availability of mean BP/RP spectrum in sampled form
    /// for this source
    #[serde(deserialize_with = "invalid_option")]
    pub has_xp_sampled: Option<bool>,

    /// Flag indicating the availability of mean RVS spectrum for this source
    #[serde(deserialize_with = "invalid_option")]
    pub has_rvs: Option<bool>,

    /// Flag indicating the availability of epoch photometry for this source
    #[serde(deserialize_with = "invalid_option")]
    pub has_epoch_photometry: Option<bool>,

    /// Flag indicating the availability of epoch radial velocity for this
    /// source
    #[serde(deserialize_with = "invalid_option")]
    pub has_epoch_rv: Option<bool>,

    /// Flag indicating the availability of GSP-Phot MCMC samples for this
    /// source
    #[serde(deserialize_with = "invalid_option")]
    pub has_mcmc_gspphot: Option<bool>,

    /// Flag indicating the availability of MSC MCMC samples for this source
    #[serde(deserialize_with = "invalid_option")]
    pub has_mcmc_msc: Option<bool>,

    /// Flag indicating that the source is present in the Gaia Andromeda
    /// Photometric Survey (GAPS)
    #[serde(deserialize_with = "invalid_option")]
    pub in_andromeda_survey: Option<bool>,

    /// Probability from DSC-Combmod of being a quasar (data used: BP/RP
    /// spectrum, photometry, astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_combmod_quasar: Option<f32>,

    /// Probability from DSC-Combmod of being a galaxy (data used: BP/RP
    /// spectrum, photometry, astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_combmod_galaxy: Option<f32>,

    /// Probability from DSC-Combmod of being a single star (but not a white
    /// dwarf) (data used: BP/RP spectrum, photometry, astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_combmod_star: Option<f32>,

    /// Effective temperature from GSP-Phot Aeneas best library using BP/RP
    /// spectra
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspphot: Option<f32>,

    /// Lower confidence level (16%) of effective temperature from GSP-Phot
    /// Aeneas best library using BP/RP spectra
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of effective temperature from GSP-Phot
    /// Aeneas best library using BP/RP spectra
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspphot_upper: Option<f32>,

    /// Surface gravity from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspphot: Option<f32>,

    /// Lower confidence level (16%) of surface gravity from GSP-Phot Aeneas
    /// best library using BP/RP spectra
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of surface gravity from GSP-Phot Aeneas
    /// best library using BP/RP spectra
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspphot_upper: Option<f32>,

    /// Iron abundance from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspphot: Option<f32>,

    /// Lower confidence level (16%) of iron abundance from GSP-Phot Aeneas best
    /// library using BP/RP spectra
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of iron abundance from GSP-Phot Aeneas best
    /// library using BP/RP spectra
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspphot_upper: Option<f32>,

    /// Distance from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_gspphot: Option<f32>,

    /// Lower confidence level (16%) of distance from GSP-Phot Aeneas best
    /// library using BP/RP spectra
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of distance from GSP-Phot Aeneas best
    /// library using BP/RP spectra
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_gspphot_upper: Option<f32>,

    /// Monochromatic extinction $A_0$ at 547.7nm from GSP-Phot Aeneas best
    /// library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_gspphot: Option<f32>,

    /// Lower confidence level (16%) of monochromatic extinction $A_0$ at
    /// 547.7nm from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of monochromatic extinction $A_0$ at
    /// 547.7nm from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_gspphot_upper: Option<f32>,

    /// Extinction in G band from GSP-Phot Aeneas best library using BP/RP
    /// spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_gspphot: Option<f32>,

    /// Lower confidence level (16%) of extinction in G band from GSP-Phot
    /// Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of extinction in G band from GSP-Phot
    /// Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_gspphot_upper: Option<f32>,

    /// Reddening $E(G_{\rm BP} - G_{\rm RP})$ from GSP-Phot Aeneas best library
    /// using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ebpminrp_gspphot: Option<f32>,

    /// Lower confidence level (16%) of reddening  $E(G_{\rm BP} - G_{\rm RP})$
    /// from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ebpminrp_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of reddening  $E(G_{\rm BP} - G_{\rm RP})$
    /// from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ebpminrp_gspphot_upper: Option<f32>,

    /// Name of library that achieves the highest mean log-posterior in MCMC
    /// samples and was used to derive GSP-Phot parameters in this table
    #[serde(deserialize_with = "invalid_option")]
    pub libname_gspphot: Option<String>,
}
