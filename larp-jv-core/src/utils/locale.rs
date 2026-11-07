//! Locale and date utilities

use crate::error::LarpResult;
use chrono::{DateTime, Utc, TimeZone, Local, Datelike, Timelike};
use std::collections::HashMap;

/// Supported languages (ISO 639-1 + ISO 3166-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Locale {
    pub language: &'static str,
    pub country: Option<&'static str>,
    pub variant: Option<&'static str>,
}

impl Locale {
    pub const fn new(language: &'static str) -> Self {
        Self { language, country: None, variant: None }
    }

    pub const fn with_country(language: &'static str, country: &'static str) -> Self {
        Self { language, country: Some(country), variant: None }
    }

    pub const fn with_variant(language: &'static str, country: &'static str, variant: &'static str) -> Self {
        Self { language, country: Some(country), variant: Some(variant) }
    }

    pub fn to_string(&self) -> String {
        match (self.country, self.variant) {
            (Some(c), Some(v)) => format!("{}_{}_{}", self.language, c, v),
            (Some(c), None) => format!("{}_{}", self.language, c),
            (None, _) => self.language.to_string(),
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('_').collect();
        match parts.len() {
            1 => Some(Self::new(parts[0])),
            2 => Some(Self::with_country(parts[0], parts[1])),
            3 => Some(Self::with_variant(parts[0], parts[1], parts[2])),
            _ => None,
        }
    }
}

/// Common locales
impl Locale {
    pub const EN_US: Locale = Locale::with_country("en", "US");
    pub const EN_GB: Locale = Locale::with_country("en", "GB");
    pub const PT_BR: Locale = Locale::with_country("pt", "BR");
    pub const PT_PT: Locale = Locale::with_country("pt", "PT");
    pub const ES_ES: Locale = Locale::with_country("es", "ES");
    pub const ES_MX: Locale = Locale::with_country("es", "MX");
    pub const FR_FR: Locale = Locale::with_country("fr", "FR");
    pub const DE_DE: Locale = Locale::with_country("de", "DE");
    pub const IT_IT: Locale = Locale::with_country("it", "IT");
    pub const JA_JP: Locale = Locale::with_country("ja", "JP");
    pub const KO_KR: Locale = Locale::with_country("ko", "KR");
    pub const ZH_CN: Locale = Locale::with_country("zh", "CN");
    pub const ZH_TW: Locale = Locale::with_country("zh", "TW");
    pub const RU_RU: Locale = Locale::with_country("ru", "RU");
    pub const PL_PL: Locale = Locale::with_country("pl", "PL");
    pub const TR_TR: Locale = Locale::with_country("tr", "TR");
    pub const AR_SA: Locale = Locale::with_country("ar", "SA");
    pub const HI_IN: Locale = Locale::with_country("hi", "IN");
    pub const TH_TH: Locale = Locale::with_country("th", "TH");
    pub const VI_VN: Locale = Locale::with_country("vi", "VN");
    pub const ID_ID: Locale = Locale::with_country("id", "ID");
    pub const MS_MY: Locale = Locale::with_country("ms", "MY");
    pub const TL_PH: Locale = Locale::with_country("tl", "PH");
    pub const NL_NL: Locale = Locale::with_country("nl", "NL");
    pub const SV_SE: Locale = Locale::with_country("sv", "SE");
    pub const DA_DK: Locale = Locale::with_country("da", "DK");
    pub const NO_NO: Locale = Locale::with_country("no", "NO");
    pub const FI_FI: Locale = Locale::with_country("fi", "FI");
    pub const CS_CZ: Locale = Locale::with_country("cs", "CZ");
    pub const SK_SK: Locale = Locale::with_country("sk", "SK");
    pub const HU_HU: Locale = Locale::with_country("hu", "HU");
    pub const RO_RO: Locale = Locale::with_country("ro", "RO");
    pub const BG_BG: Locale = Locale::with_country("bg", "BG");
    pub const HR_HR: Locale = Locale::with_country("hr", "HR");
    pub const SL_SI: Locale = Locale::with_country("sl", "SI");
    pub const ET_EE: Locale = Locale::with_country("et", "EE");
    pub const LV_LV: Locale = Locale::with_country("lv", "LV");
    pub const LT_LT: Locale = Locale::with_country("lt", "LT");
    pub const EL_GR: Locale = Locale::with_country("el", "GR");
    pub const HE_IL: Locale = Locale::with_country("he", "IL");
    pub const FA_IR: Locale = Locale::with_country("fa", "IR");
    pub const UK_UA: Locale = Locale::with_country("uk", "UA");
    pub const BE_BY: Locale = Locale::with_country("be", "BY");
    pub const KA_GE: Locale = Locale::with_country("ka", "GE");
    pub const AZ_AZ: Locale = Locale::with_country("az", "AZ");
    pub const KK_KZ: Locale = Locale::with_country("kk", "KZ");
    pub const KY_KG: Locale = Locale::with_country("ky", "KG");
    pub const UZ_UZ: Locale = Locale::with_country("uz", "UZ");
    pub const MN_MN: Locale = Locale::with_country("mn", "MN");
}

/// Get system locale
pub fn system_locale() -> Locale {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::GetUserDefaultLocaleName;
        let mut buf = [0u16; 85];
        let len = unsafe { GetUserDefaultLocaleName(&mut buf) } as usize;
        if len > 0 {
            let s = String::from_utf16_lossy(&buf[..len]);
            return Locale::parse(&s.replace('-', "_")).unwrap_or(Locale::EN_US);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(lang) = std::env::var("LANG") {
            if let Some(locale) = Locale::parse(&lang.split('.').next().unwrap_or("").replace('.', "_")) {
                return locale;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(lang) = std::env::var("LANG") {
            if let Some(locale) = Locale::parse(&lang.split('.').next().unwrap_or("").replace('.', "_")) {
                return locale;
            }
        }
    }
    Locale::EN_US
}

/// Format date/time for locale
pub fn format_datetime(dt: DateTime<Utc>, locale: &Locale) -> String {
    let local_dt = dt.with_timezone(&Local);
    match locale.language {
        "en" => local_dt.format("%m/%d/%Y %I:%M %p").to_string(),
        "pt" => local_dt.format("%d/%m/%Y %H:%M").to_string(),
        "es" | "fr" | "de" | "it" => local_dt.format("%d.%m.%Y %H:%M").to_string(),
        "ja" => local_dt.format("%Y/%m/%d %H:%M").to_string(),
        "ko" => local_dt.format("%Y.%m.%d %H:%M").to_string(),
        "zh" => local_dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => local_dt.format("%Y-%m-%d %H:%M").to_string(),
    }
}

/// Format relative time (e.g., "2 hours ago")
pub fn format_relative(dt: DateTime<Utc>, locale: &Locale) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        match locale.language {
            "en" => "just now".to_string(),
            "pt" => "agora mesmo".to_string(),
            "es" => "justo ahora".to_string(),
            "fr" => "à l'instant".to_string(),
            "de" => "gerade eben".to_string(),
            "ja" => "たった今".to_string(),
            "ko" => "방금 전".to_string(),
            "zh" => "刚刚".to_string(),
            _ => "just now".to_string(),
        }
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        match locale.language {
            "en" => format!("{} min ago", mins),
            "pt" => format!("{} min atrás", mins),
            "es" => format!("hace {} min", mins),
            "fr" => format!("il y a {} min", mins),
            "de" => format!("vor {} Min", mins),
            "ja" => format!("{}分前", mins),
            "ko" => format!("{}분 전", mins),
            "zh" => format!("{}分钟前", mins),
            _ => format!("{} min ago", mins),
        }
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        match locale.language {
            "en" => format!("{} hours ago", hours),
            "pt" => format!("{} horas atrás", hours),
            "es" => format!("hace {} horas", hours),
            "fr" => format!("il y a {} heures", hours),
            "de" => format!("vor {} Stunden", hours),
            "ja" => format!("{}時間前", hours),
            "ko" => format!("{}시간 전", hours),
            "zh" => format!("{}小时前", hours),
            _ => format!("{} hours ago", hours),
        }
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        match locale.language {
            "en" => format!("{} days ago", days),
            "pt" => format!("{} dias atrás", days),
            "es" => format!("hace {} días", days),
            "fr" => format!("il y a {} jours", days),
            "de" => format!("vor {} Tagen", days),
            "ja" => format!("{}日前", days),
            "ko" => format!("{}일 전", days),
            "zh" => format!("{}天前", days),
            _ => format!("{} days ago", days),
        }
    } else {
        format_datetime(dt, locale)
    }
}

/// Number formatting for locale
pub fn format_number(num: f64, locale: &Locale) -> String {
    match locale.language {
        "en" => format!("{:.2}", num).replace(",", "_").replace(".", ",").replace("_", "."),
        "de" | "fr" | "es" | "it" | "pt" => format!("{:.2}", num).replace(".", ","),
        "ja" | "ko" | "zh" => format!("{:.2}", num),
        _ => format!("{:.2}", num),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_parsing() {
        assert_eq!(Locale::parse("en_US").unwrap().to_string(), "en_US");
        assert_eq!(Locale::parse("pt_BR").unwrap().to_string(), "pt_BR");
        assert_eq!(Locale::parse("zh_CN").unwrap().to_string(), "zh_CN");
    }

    #[test]
    fn test_format_relative() {
        let now = Utc::now();
        let locale = Locale::EN_US;
        assert_eq!(format_relative(now, &locale), "just now");
        assert_eq!(format_relative(now - chrono::Duration::minutes(5), &locale), "5 min ago");
        assert_eq!(format_relative(now - chrono::Duration::hours(2), &locale), "2 hours ago");
        assert_eq!(format_relative(now - chrono::Duration::days(3), &locale), "3 days ago");
    }
}