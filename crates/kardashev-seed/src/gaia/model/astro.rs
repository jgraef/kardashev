use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::invalid_option;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstrophysicalParameters {
    /// Solution Identifier
    pub solution_id: u64,

    /// Source Identifier
    pub source_id: u64,

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

    /// Probability from DSC-Combmod of being a white dwarf (data used: BP/RP
    /// spectrum, photometry, astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_combmod_whitedwarf: Option<f32>,

    /// Probability from DSC-Combmod of being a binary star (data used: BP/RP
    /// spectrum, photometry, astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_combmod_binarystar: Option<f32>,

    /// Probability from DSC-Specmod of being a quasar (data used: BP/RP
    /// spectrum)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_specmod_quasar: Option<f32>,

    /// Probability from DSC-Specmod of being a galaxy (data used: BP/RP
    /// spectrum)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_specmod_galaxy: Option<f32>,

    /// Probability from DSC-Specmod of being a single star (but not a white
    /// dwarf) (data used: BP/RP spectrum)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_specmod_star: Option<f32>,

    /// Probability from DSC-Specmod of being a white dwarf (data used: BP/RP
    /// spectrum)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_specmod_whitedwarf: Option<f32>,

    /// Probability from DSC-Specmod of being a binary star (data used: BP/RP
    /// spectrum)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_specmod_binarystar: Option<f32>,

    /// Probability from DSC-Allosmod of being a quasar (data used: photometry,
    /// astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_allosmod_quasar: Option<f32>,

    /// Probability from DSC-Allosmod of being a galaxy (data used: photometry,
    /// astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_allosmod_galaxy: Option<f32>,

    /// Probability from DSC-Allosmod of being a star (data used: photometry,
    /// astrometry)
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_dsc_allosmod_star: Option<f32>,

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

    /// Extinction in $G_{\rm BP}$ band from GSP-Phot Aeneas best library using
    /// BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub abp_gspphot: Option<f32>,

    /// Lower confidence level (16%) of extinction in $G_{\rm BP}$ band from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub abp_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of extinction in $G_{\rm BP}$ band from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub abp_gspphot_upper: Option<f32>,

    /// Extinction in $G_{\rm RP}$ band from GSP-Phot Aeneas best library using
    /// BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub arp_gspphot: Option<f32>,

    /// Lower confidence level (16%) of extinction in $G_{\rm RP}$ band from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub arp_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of extinction in $G_{\rm RP}$ band from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub arp_gspphot_upper: Option<f32>,

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

    /// Absolute magnitude $M_{\rm G}$ from GSP-Phot Aeneas best library using
    /// BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub mg_gspphot: Option<f32>,

    /// Lower confidence level (16%) of absolute magnitude $M_{\rm G}$ from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub mg_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of absolute magnitude $M_{\rm G}$ from
    /// GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub mg_gspphot_upper: Option<f32>,

    /// Radius from GSP-Phot Aeneas best library using BP/RP spectra
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_gspphot: Option<f32>,

    /// Lower confidence level (16%) of radius from GSP-Phot Aeneas best library
    /// using BP/RP spectra
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_gspphot_lower: Option<f32>,

    /// Upper confidence level (84%) of radius from GSP-Phot Aeneas best library
    /// using BP/RP spectra
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_gspphot_upper: Option<f32>,

    /// Goodness-of-fit score (mean log-posterior of MCMC) of GSP-Phot Aeneas
    /// MCMC best library
    #[serde(deserialize_with = "invalid_option")]
    pub logposterior_gspphot: Option<f32>,

    /// MCMC acceptance rate of GSP-Phot Aeneas MCMC best library
    #[serde(deserialize_with = "invalid_option")]
    pub mcmcaccept_gspphot: Option<f32>,

    /// Name of library that achieves the highest mean log-posterior in MCMC
    /// samples and was used to derive GSP-Phot parameters in this table
    #[serde(deserialize_with = "invalid_option")]
    pub libname_gspphot: Option<String>,

    /// Effective temperature from GSP-Spec MatisseGauguin using RVS spectra and
    /// Monte Carlo realisations
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspspec: Option<f32>,

    /// 16th percentile of effective temperature from GSP-Spec MatisseGauguin
    /// using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspspec_lower: Option<f32>,

    /// 84th percentile of effective temperature from GSP-Spec MatisseGauguin
    /// using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_gspspec_upper: Option<f32>,

    /// Logarithm of the stellar surface gravity from GSP-Spec MatisseGauguin
    /// using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspspec: Option<f32>,

    /// 16th percentile of the logarithm of the stellar surface gravity from
    /// GSP-Spec MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspspec_lower: Option<f32>,

    /// 84th percentile of the logarithm of the stellar surface gravity from
    /// GSP-Spec MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_gspspec_upper: Option<f32>,

    /// Global metallicity [M/H] from GSP-Spec MatisseGauguin using RVS spectra
    /// and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspspec: Option<f32>,

    /// 16th percentile of global metallicity [M/H] from GSP-Spec MatisseGauguin
    /// using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspspec_lower: Option<f32>,

    /// 84th percentile of global metallicity [M/H] from GSP-Spec MatisseGauguin
    /// using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_gspspec_upper: Option<f32>,

    /// Abundance of alpha-elements [alpha/Fe] with respect to iron from
    /// GSP-Spec MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub alphafe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of alpha-elements [alpha/Fe] with
    /// respect to iron from GSP-Spec MatisseGauguin using RVS spectra and Monte
    /// Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub alphafe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of alpha-elements [alpha/Fe] with
    /// respect to iron from GSP-Spec MatisseGauguin using RVS spectra and Monte
    /// Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub alphafe_gspspec_upper: Option<f32>,

    /// Abundance of neutral iron [Fe/M] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt femGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub fem_gspspec: Option<f32>,

    /// 16th percentile of the abundance of neutral iron [Fe/M] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub fem_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of neutral iron [Fe/M] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub fem_gspspec_upper: Option<f32>,

    /// Number of lines used for [Fe/M] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub fem_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Fe/M] abundance using N lines of the element,
    /// given in {\tt femGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub fem_gspspec_linescatter: Option<f32>,

    /// Abundance of Silicon [Si/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt sifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sife_gspspec: Option<f32>,

    /// 16th percentile of the abundance of silicon [Si/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sife_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of silicon [Si/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sife_gspspec_upper: Option<f32>,

    /// Number of lines used for [Si/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub sife_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Si/Fe] abundance using N lines of the
    /// element, given in {\tt sifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sife_gspspec_linescatter: Option<f32>,

    /// Abundance of Calcium [Ca/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt cafeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cafe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Calcium [Ca/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cafe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Calcium [Ca/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cafe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Ca/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub cafe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Ca/Fe] abundance using N lines of the
    /// element, given in {\tt cafeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cafe_gspspec_linescatter: Option<f32>,

    /// Abundance of Titanium [Ti/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt tifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub tife_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Titanium [Ti/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub tife_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Titanium [Ti/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub tife_gspspec_upper: Option<f32>,

    /// Number of lines used for [Ti/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub tife_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Ti/Fe] abundance using N lines of the
    /// element, given in {\tt tifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub tife_gspspec_linescatter: Option<f32>,

    /// Abundance of Magnesium [Mg/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt mgfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mgfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Magnesium [Mg/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mgfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Magnesium [Mg/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mgfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Mg/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub mgfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Mg/Fe] abundance using N lines of the
    /// element, given in {\tt mgfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mgfe_gspspec_linescatter: Option<f32>,

    /// Abundance of neodymium [Nd/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt ndfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub ndfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of neodymium [Nd/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub ndfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of neodymium [Nd/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub ndfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Nd/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub ndfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Nd/Fe] abundance using N lines of the
    /// element, given in {\tt ndfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub ndfe_gspspec_linescatter: Option<f32>,

    /// Abundance of ionised iron [FeII/M] from GSP-Spec MatisseGauguin using
    /// RVS spectra and Monte Carlo realisations, applied to the individual N
    /// lines of the element, given in {\tt feiimGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub feiim_gspspec: Option<f32>,

    /// 16th percentile of the abundance of ionised iron [FeII/M] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub feiim_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of ionised iron [FeII/M] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub feiim_gspspec_upper: Option<f32>,

    /// Number of lines used for [FeII/M] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub feiim_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [FeII/M] abundance using N lines of the
    /// element, given in {\tt feiimGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub feiim_gspspec_linescatter: Option<f32>,

    /// Abundance of Sulphur [S/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt sfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Sulphur [S/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Sulphur [S/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [S/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub sfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [S/Fe] abundance using N lines of the element,
    /// given in {\tt sfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub sfe_gspspec_linescatter: Option<f32>,

    /// Abundance of Zirconium [Zr/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt zrfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub zrfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Zirconium [Zr/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub zrfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Zirconium [Zr/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub zrfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Zr/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub zrfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Zr/Fe] abundance using N lines of the
    /// element, given in {\tt zrfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub zrfe_gspspec_linescatter: Option<f32>,

    /// Abundance of Nitrogen [N/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt nfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Nitrogen [N/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Nitrogen [N/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [N/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub nfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [N/Fe] abundance using N lines of the element,
    /// given in {\tt nfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nfe_gspspec_linescatter: Option<f32>,

    /// Abundance of Chromium [Cr/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt crfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub crfe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Chromium [Cr/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub crfe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Chromium [Cr/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub crfe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Cr/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub crfe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Cr/Fe] abundance using N lines of the
    /// element, given in {\tt crfeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub crfe_gspspec_linescatter: Option<f32>,

    /// Abundance of Cerium [Ce/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt cefeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cefe_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Cerium [Ce/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cefe_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Cerium [Ce/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cefe_gspspec_upper: Option<f32>,

    /// Number of lines used for [Ce/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub cefe_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Ce/Fe] abundance using N lines of the
    /// element, given in {\tt cefeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub cefe_gspspec_linescatter: Option<f32>,

    /// Abundance of Nickel [Ni/Fe] from GSP-Spec MatisseGauguin using RVS
    /// spectra and Monte Carlo realisations, applied to the individual N lines
    /// of the element, given in {\tt nifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nife_gspspec: Option<f32>,

    /// 16th percentile of the abundance of Nickel [Ni/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nife_gspspec_lower: Option<f32>,

    /// 84th percentile of the abundance of Nickel [Ni/Fe] from GSP-Spec
    /// MatisseGauguin using RVS spectra and Monte Carlo realisations
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nife_gspspec_upper: Option<f32>,

    /// Number of lines used for [Ni/Fe] abundance estimation
    #[serde(deserialize_with = "invalid_option")]
    pub nife_gspspec_nlines: Option<i32>,

    /// Uncertainty estimation of [Ni/Fe] abundance using N lines of the
    /// element, given in {\tt nifeGspspecNlines}
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub nife_gspspec_linescatter: Option<f32>,

    /// Equivalent witdh of cyanogen absorption line, derived from RVS spectra
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub cn0ew_gspspec: Option<f32>,

    /// Uncertainty of equivalent witdh of cyanogen absorption line, derived
    /// from RVS spectra
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub cn0ew_gspspec_uncertainty: Option<f32>,

    /// Central wavelength of cyanogen line, derived from RVS spectra using DIB
    /// algorithm
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub cn0_gspspec_centralline: Option<f32>,

    /// Width of cyoanogen line, derived from RVS spectra using DIB algorithm
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub cn0_gspspec_width: Option<f32>,

    /// DIB central wavelength from GSP-Spec MatisseGauguin using RVS spectra
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub dib_gspspec_lambda: Option<f32>,

    /// Uncertainty on DIB central wavelength from GSP-Spec MatisseGauguin using
    /// RVS spectra
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub dib_gspspec_lambda_uncertainty: Option<f32>,

    /// Equivalent width of the DIB from GSP-Spec MatisseGauguin using RVS
    /// spectra
    ///
    /// Unit: angstrom
    #[serde(deserialize_with = "invalid_option")]
    pub dibew_gspspec: Option<f32>,

    /// Global uncertainty on DIB equivalent width value using DIB algorithm
    ///
    /// Unit: angstrom
    #[serde(deserialize_with = "invalid_option")]
    pub dibew_gspspec_uncertainty: Option<f32>,

    /// Uncertainty on DIB equivalent width value occuring from noise part
    ///
    /// Unit: angstrom
    #[serde(deserialize_with = "invalid_option")]
    pub dibewnoise_gspspec_uncertainty: Option<f32>,

    /// Depth ($p_0$ parameter) of the DIB derived from a Gaussian model fit
    #[serde(deserialize_with = "invalid_option")]
    pub dibp0_gspspec: Option<f32>,

    /// Width ($p_2$ parameter) of the DIB derived from a Gaussian model fit
    ///
    /// Unit: angstrom
    #[serde(deserialize_with = "invalid_option")]
    pub dibp2_gspspec: Option<f32>,

    /// Uncertainty on the {\tt dibp2Gspspec} parameter
    ///
    /// Unit: angstrom
    #[serde(deserialize_with = "invalid_option")]
    pub dibp2_gspspec_uncertainty: Option<f32>,

    /// Quality flag of the DIB computation
    #[serde(deserialize_with = "invalid_option")]
    pub dibqf_gspspec: Option<i32>,

    /// Catalogue flags for GSP-Spec MatisseGauguin
    #[serde(deserialize_with = "invalid_option")]
    pub flags_gspspec: Option<String>,

    /// Logarithm of the goodness-of-fit for the GSP-Spec MatisseGauguin
    /// parameters
    #[serde(deserialize_with = "invalid_option")]
    pub logchisq_gspspec: Option<f32>,

    /// Halpha pseudo-equivalent width from ESP-ELS
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub ew_espels_halpha: Option<f32>,

    /// Uncertainty of the Halpha pseudo-equivalent width from ESP-ELS
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub ew_espels_halpha_uncertainty: Option<f32>,

    /// Quality flag of the Halpha pseudo-equivalent width from ESP-ELS
    #[serde(deserialize_with = "invalid_option")]
    pub ew_espels_halpha_flag: Option<String>,

    /// Halpha pseudo-equivalent width from ESP-ELS measured on the synthetic
    /// spectrum
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub ew_espels_halpha_model: Option<f32>,

    /// Adopted ELS class label from ESP-ELS
    #[serde(deserialize_with = "invalid_option")]
    pub classlabel_espels: Option<String>,

    /// Quality flag of the adopted ELS class label from ESP-ELS
    #[serde(deserialize_with = "invalid_option")]
    pub classlabel_espels_flag: Option<String>,

    /// Probability from ESP-ELS of being a Wolf-Rayet star of type WC
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_wcstar: Option<f32>,

    /// Probability from ESP-ELS of being a Wolf-Rayet star of type WN
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_wnstar: Option<f32>,

    /// Probability from ESP-ELS of being a Be Star
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_bestar: Option<f32>,

    /// Probability from ESP-ELS of being a T Tauri Star
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_ttauristar: Option<f32>,

    /// Probability from ESP-ELS of being a Herbig Ae/Be Star
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_herbigstar: Option<f32>,

    /// Probability from ESP-ELS of being an active M dwarf Star
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_dmestar: Option<f32>,

    /// Probability from ESP-ELS of being a planetary nebula
    #[serde(deserialize_with = "invalid_option")]
    pub classprob_espels_pne: Option<f32>,

    /// Monochromatic interstellar extinction, A$_\mathrm{0}$, from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_esphs: Option<f32>,

    /// Uncertainty at a 68% confidence level on A$_\mathrm{0}$ from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_esphs_uncertainty: Option<f32>,

    /// Intersterstellar extinction in G band from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_esphs: Option<f32>,

    /// Uncertainty on $A_{\rm G}$ from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_esphs_uncertainty: Option<f32>,

    /// Reddening $E(G_{\rm BP} - G_{\rm RP})$ from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ebpminrp_esphs: Option<f32>,

    /// Uncertainty on $E(G_{\rm BP} - G_{\rm RP})$ from ESP-HS
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ebpminrp_esphs_uncertainty: Option<f32>,

    /// Effective temperature from ESP-HS
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_esphs: Option<f32>,

    /// Uncertainty at a 68% confidence level on the effective temperature from
    /// ESP-HS
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_esphs_uncertainty: Option<f32>,

    /// Surface gravity from ESP-HS
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_esphs: Option<f32>,

    /// Uncertainty at a 68% confidence level on the surface gravity from ESP-HS
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_esphs_uncertainty: Option<f32>,

    /// Projected rotational velocity from ESP-HS
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub vsini_esphs: Option<f32>,

    /// Uncertainty on the projected rotational velocity from ESP-HS
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub vsini_esphs_uncertainty: Option<f32>,

    /// Quality flag of the ESP-HS parametrisation
    #[serde(deserialize_with = "invalid_option")]
    pub flags_esphs: Option<String>,

    /// Spectral type from ESP-ELS
    #[serde(deserialize_with = "invalid_option")]
    pub spectraltype_esphs: Option<String>,

    /// Chromospheric activity index from ESP-CS, measured on the calcium
    /// triplet using RVS spectra
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub activityindex_espcs: Option<f32>,

    /// Uncertainty in the chromospheric activity index from ESP-CS
    ///
    /// Unit: nm
    #[serde(deserialize_with = "invalid_option")]
    pub activityindex_espcs_uncertainty: Option<f32>,

    /// Source of input stellar parameters for the computation of the activity
    /// index by ESP-CS
    #[serde(deserialize_with = "invalid_option")]
    pub activityindex_espcs_input: Option<String>,

    /// Effective temperature estimate from ESP-UCD based on the RP spectrum
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_espucd: Option<f32>,

    /// Uncertainty of the effective temperature estimate produced by ESP-UCD
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_espucd_uncertainty: Option<f32>,

    /// Quality flags of the ESP-UCD parameter estimates
    #[serde(deserialize_with = "invalid_option")]
    pub flags_espucd: Option<String>,

    /// Radius of the star from FLAME using {\tt teffGspphot} and {\tt lumFlame}
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_flame: Option<f32>,

    /// Lower confidence level (16%) of {\tt radiusFlame}
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_flame_lower: Option<f32>,

    /// Upper confidence level (84%) of {\tt radiusFlame}
    ///
    /// Unit: solRad
    #[serde(deserialize_with = "invalid_option")]
    pub radius_flame_upper: Option<f32>,

    /// Luminosity of the star from FLAME using G band magnitude, extinction
    /// ({\tt agGspphot}),  parallax or distance, and a bolometric correction
    /// {\tt bcFlame}
    ///
    /// Unit: solLum
    #[serde(deserialize_with = "invalid_option")]
    pub lum_flame: Option<f32>,

    /// Lower confidence level (16%) of {\tt lumFlame}
    ///
    /// Unit: solLum
    #[serde(deserialize_with = "invalid_option")]
    pub lum_flame_lower: Option<f32>,

    /// Upper confidence level (84%) of {\tt lumFlame}
    ///
    /// Unit: solLum
    #[serde(deserialize_with = "invalid_option")]
    pub lum_flame_upper: Option<f32>,

    /// Mass of the star from FLAME using stellar models, {\tt lumFlame}, and
    /// {\tt teffGspphot}
    ///
    /// Unit: solMass
    #[serde(deserialize_with = "invalid_option")]
    pub mass_flame: Option<f32>,

    /// Lower confidence level (16%) of {\tt massFlame}
    ///
    /// Unit: solMass
    #[serde(deserialize_with = "invalid_option")]
    pub mass_flame_lower: Option<f32>,

    /// Upper confidence level (84%) of {\tt massFlame}
    ///
    /// Unit: solMass
    #[serde(deserialize_with = "invalid_option")]
    pub mass_flame_upper: Option<f32>,

    /// Age of the star from FLAME using stellar models, see {\tt massFlame} for
    /// details
    ///
    /// Unit: Gyr
    #[serde(deserialize_with = "invalid_option")]
    pub age_flame: Option<f32>,

    /// Lower confidence level (16%) of {\tt ageFlame}
    ///
    /// Unit: Gyr
    #[serde(deserialize_with = "invalid_option")]
    pub age_flame_lower: Option<f32>,

    /// Upper confidence level (84%) of {\tt ageFlame}
    ///
    /// Unit: Gyr
    #[serde(deserialize_with = "invalid_option")]
    pub age_flame_upper: Option<f32>,

    /// Flags indicating quality and processing information from FLAME
    #[serde(deserialize_with = "invalid_option")]
    pub flags_flame: Option<String>,

    /// Evolutionary stage of the star from FLAME using stellar models, see {\tt
    /// massFlame} for details
    #[serde(deserialize_with = "invalid_option")]
    pub evolstage_flame: Option<i32>,

    /// Gravitational redshift from FLAME using {\tt radiusFlame} and {\tt
    /// loggGspphot}
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub gravredshift_flame: Option<f32>,

    /// Lower confidence level (16%) of {\tt gravredshiftFlame}
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub gravredshift_flame_lower: Option<f32>,

    /// Upper confidence level (84%) of {\tt gravredshiftFlame}
    ///
    /// Unit: km.s**-1
    #[serde(deserialize_with = "invalid_option")]
    pub gravredshift_flame_upper: Option<f32>,

    /// Bolometric correction used to derive {\tt lumFlame}
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub bc_flame: Option<f32>,

    /// Metallicity of the source treated as a binary system from MSC using
    /// BP/RP spectra and parallax
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_msc: Option<f32>,

    /// Upper confidence level (84%) of the metallicity from MSC using BP/RP
    /// spectra and parallax
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_msc_upper: Option<f32>,

    /// Lower confidence level (16%) of the metallicity from MSC using BP/RP
    /// spectra and parallax
    ///
    /// Unit: 'dex'
    #[serde(deserialize_with = "invalid_option")]
    pub mh_msc_lower: Option<f32>,

    /// Monochromatic extinction $A_0$ at 547.7nm of the source treated as a
    /// binary system from MSC using BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_msc: Option<f32>,

    /// Upper confidence level (84%) of monochromatic extinction $A_0$ at
    /// 547.7nm from MSC using BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_msc_upper: Option<f32>,

    /// Lower confidence level (16%) of monochromatic extinction $A_0$ at
    /// 547.7nm from MSC using BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub azero_msc_lower: Option<f32>,

    /// Distance from MSC using BP/RP spectra and parallax
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_msc: Option<f32>,

    /// Upper confidence level (84%) of distance from MSC using BP/RP spectra
    /// and parallax
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_msc_upper: Option<f32>,

    /// Lower confidence level (16%) of distance from MSC using BP/RP spectra
    /// and parallax
    ///
    /// Unit: pc
    #[serde(deserialize_with = "invalid_option")]
    pub distance_msc_lower: Option<f32>,

    /// Effective temperature of the primary from MSC using BP/RP spectra and
    /// parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc1: Option<f32>,

    /// Upper confidence level (84%) of effective temperature of the primary
    /// from MSC using BP/RP spectra and parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc1_upper: Option<f32>,

    /// Lower confidence level (16%) of effective temperature of the primary
    /// from MSC using BP/RP spectra and parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc1_lower: Option<f32>,

    /// Effective temperature of the secondary from MSC using BP/RP spectra and
    /// parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc2: Option<f32>,

    /// Upper confidence level (84%) of effective temperature of the secondary
    /// from MSC using BP/RP spectra and parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc2_upper: Option<f32>,

    /// Lower confidence level (16%) of effective temperature of the secondary
    /// from MSC using BP/RP spectra and parallax
    ///
    /// Unit: K
    #[serde(deserialize_with = "invalid_option")]
    pub teff_msc2_lower: Option<f32>,

    /// Surface gravity of the primary from MSC using BP/RP spectra and parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc1: Option<f32>,

    /// Upper confidence level (84%) of surface gravity of the primary from MSC
    /// using BP/RP spectra and parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc1_upper: Option<f32>,

    /// Lower confidence level (16%) of surface gravity of the primary from MSC
    /// using BP/RP spectra and parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc1_lower: Option<f32>,

    /// Surface gravity of the secondary from MSC using BP/RP spectra and
    /// parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc2: Option<f32>,

    /// Upper confidence level (84%) of surface gravity of the secondary from
    /// MSC using BP/RP spectra and parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc2_upper: Option<f32>,

    /// Lower confidence level (16%) of surface gravity of the secondary from
    /// MSC using BP/RP spectra and parallax
    ///
    /// Unit: log(cm.s**-2)
    #[serde(deserialize_with = "invalid_option")]
    pub logg_msc2_lower: Option<f32>,

    /// Extinction in G band of the source treated as a binary system from MSC
    /// using BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_msc: Option<f32>,

    /// Upper confidence level (84%) of extinction in G band from MSC using
    /// BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_msc_upper: Option<f32>,

    /// Lower confidence level (16%) of extinction in G band from MSC using
    /// BP/RP spectra and parallax
    ///
    /// Unit: mag
    #[serde(deserialize_with = "invalid_option")]
    pub ag_msc_lower: Option<f32>,

    /// Goodness-of-fit score (normalised log-posterior) of MSC MCMC
    #[serde(deserialize_with = "invalid_option")]
    pub logposterior_msc: Option<f32>,

    /// Mean MCMC acceptance rate of MSC MCMC
    #[serde(deserialize_with = "invalid_option")]
    pub mcmcaccept_msc: Option<f32>,

    /// Mean drift of the MSC MCMC chain in units of parameter standard
    /// deviation
    #[serde(deserialize_with = "invalid_option")]
    pub mcmcdrift_msc: Option<f32>,

    /// Flag indicating quality information from MSC
    #[serde(deserialize_with = "invalid_option")]
    pub flags_msc: Option<String>,

    /// Identifier of the OA SOM map neuron that represents the source
    #[serde(deserialize_with = "invalid_option")]
    pub neuron_oa_id: Option<i64>,

    /// Distance between the source XP spectra and the OA neuron XP prototype
    /// that represents the source
    #[serde(deserialize_with = "invalid_option")]
    pub neuron_oa_dist: Option<f32>,

    /// Percentile rank according to the distance distribution of the OA neuron
    /// that represents the source
    #[serde(deserialize_with = "invalid_option")]
    pub neuron_oa_dist_percentile_rank: Option<i32>,

    /// Flags indicating quality and processing information from OA
    #[serde(deserialize_with = "invalid_option")]
    pub flags_oa: Option<String>,
}
