//! Countries with their international dialling codes, for the phone number
//! a buyer leaves before paying (so the site owner can get back to them).

/// (ISO 3166-1 alpha-2, dialling code without "+", English name)
pub const COUNTRIES: &[(&str, &str, &str)] = &[
    ("IL", "972", "Israel"),
    ("US", "1", "United States"),
    ("GB", "44", "United Kingdom"),
    ("CA", "1", "Canada"),
    ("FR", "33", "France"),
    ("DE", "49", "Germany"),
    ("RU", "7", "Russia"),
    ("UA", "380", "Ukraine"),
    ("AF", "93", "Afghanistan"),
    ("AL", "355", "Albania"),
    ("DZ", "213", "Algeria"),
    ("AD", "376", "Andorra"),
    ("AO", "244", "Angola"),
    ("AG", "1", "Antigua and Barbuda"),
    ("AR", "54", "Argentina"),
    ("AM", "374", "Armenia"),
    ("AU", "61", "Australia"),
    ("AT", "43", "Austria"),
    ("AZ", "994", "Azerbaijan"),
    ("BS", "1", "Bahamas"),
    ("BH", "973", "Bahrain"),
    ("BD", "880", "Bangladesh"),
    ("BB", "1", "Barbados"),
    ("BY", "375", "Belarus"),
    ("BE", "32", "Belgium"),
    ("BZ", "501", "Belize"),
    ("BJ", "229", "Benin"),
    ("BT", "975", "Bhutan"),
    ("BO", "591", "Bolivia"),
    ("BA", "387", "Bosnia and Herzegovina"),
    ("BW", "267", "Botswana"),
    ("BR", "55", "Brazil"),
    ("BN", "673", "Brunei"),
    ("BG", "359", "Bulgaria"),
    ("BF", "226", "Burkina Faso"),
    ("BI", "257", "Burundi"),
    ("KH", "855", "Cambodia"),
    ("CM", "237", "Cameroon"),
    ("CV", "238", "Cape Verde"),
    ("CF", "236", "Central African Republic"),
    ("TD", "235", "Chad"),
    ("CL", "56", "Chile"),
    ("CN", "86", "China"),
    ("CO", "57", "Colombia"),
    ("KM", "269", "Comoros"),
    ("CG", "242", "Congo"),
    ("CD", "243", "Congo (DRC)"),
    ("CR", "506", "Costa Rica"),
    ("CI", "225", "Côte d'Ivoire"),
    ("HR", "385", "Croatia"),
    ("CU", "53", "Cuba"),
    ("CY", "357", "Cyprus"),
    ("CZ", "420", "Czechia"),
    ("DK", "45", "Denmark"),
    ("DJ", "253", "Djibouti"),
    ("DM", "1", "Dominica"),
    ("DO", "1", "Dominican Republic"),
    ("EC", "593", "Ecuador"),
    ("EG", "20", "Egypt"),
    ("SV", "503", "El Salvador"),
    ("GQ", "240", "Equatorial Guinea"),
    ("ER", "291", "Eritrea"),
    ("EE", "372", "Estonia"),
    ("SZ", "268", "Eswatini"),
    ("ET", "251", "Ethiopia"),
    ("FJ", "679", "Fiji"),
    ("FI", "358", "Finland"),
    ("GA", "241", "Gabon"),
    ("GM", "220", "Gambia"),
    ("GE", "995", "Georgia"),
    ("GH", "233", "Ghana"),
    ("GR", "30", "Greece"),
    ("GD", "1", "Grenada"),
    ("GT", "502", "Guatemala"),
    ("GN", "224", "Guinea"),
    ("GW", "245", "Guinea-Bissau"),
    ("GY", "592", "Guyana"),
    ("HT", "509", "Haiti"),
    ("HN", "504", "Honduras"),
    ("HK", "852", "Hong Kong"),
    ("HU", "36", "Hungary"),
    ("IS", "354", "Iceland"),
    ("IN", "91", "India"),
    ("ID", "62", "Indonesia"),
    ("IR", "98", "Iran"),
    ("IQ", "964", "Iraq"),
    ("IE", "353", "Ireland"),
    ("IT", "39", "Italy"),
    ("JM", "1", "Jamaica"),
    ("JP", "81", "Japan"),
    ("JO", "962", "Jordan"),
    ("KZ", "7", "Kazakhstan"),
    ("KE", "254", "Kenya"),
    ("KI", "686", "Kiribati"),
    ("XK", "383", "Kosovo"),
    ("KW", "965", "Kuwait"),
    ("KG", "996", "Kyrgyzstan"),
    ("LA", "856", "Laos"),
    ("LV", "371", "Latvia"),
    ("LB", "961", "Lebanon"),
    ("LS", "266", "Lesotho"),
    ("LR", "231", "Liberia"),
    ("LY", "218", "Libya"),
    ("LI", "423", "Liechtenstein"),
    ("LT", "370", "Lithuania"),
    ("LU", "352", "Luxembourg"),
    ("MO", "853", "Macao"),
    ("MG", "261", "Madagascar"),
    ("MW", "265", "Malawi"),
    ("MY", "60", "Malaysia"),
    ("MV", "960", "Maldives"),
    ("ML", "223", "Mali"),
    ("MT", "356", "Malta"),
    ("MH", "692", "Marshall Islands"),
    ("MR", "222", "Mauritania"),
    ("MU", "230", "Mauritius"),
    ("MX", "52", "Mexico"),
    ("FM", "691", "Micronesia"),
    ("MD", "373", "Moldova"),
    ("MC", "377", "Monaco"),
    ("MN", "976", "Mongolia"),
    ("ME", "382", "Montenegro"),
    ("MA", "212", "Morocco"),
    ("MZ", "258", "Mozambique"),
    ("MM", "95", "Myanmar"),
    ("NA", "264", "Namibia"),
    ("NR", "674", "Nauru"),
    ("NP", "977", "Nepal"),
    ("NL", "31", "Netherlands"),
    ("NZ", "64", "New Zealand"),
    ("NI", "505", "Nicaragua"),
    ("NE", "227", "Niger"),
    ("NG", "234", "Nigeria"),
    ("KP", "850", "North Korea"),
    ("MK", "389", "North Macedonia"),
    ("NO", "47", "Norway"),
    ("OM", "968", "Oman"),
    ("PK", "92", "Pakistan"),
    ("PW", "680", "Palau"),
    ("PS", "970", "Palestine"),
    ("PA", "507", "Panama"),
    ("PG", "675", "Papua New Guinea"),
    ("PY", "595", "Paraguay"),
    ("PE", "51", "Peru"),
    ("PH", "63", "Philippines"),
    ("PL", "48", "Poland"),
    ("PT", "351", "Portugal"),
    ("PR", "1", "Puerto Rico"),
    ("QA", "974", "Qatar"),
    ("RO", "40", "Romania"),
    ("RW", "250", "Rwanda"),
    ("KN", "1", "Saint Kitts and Nevis"),
    ("LC", "1", "Saint Lucia"),
    ("VC", "1", "Saint Vincent and the Grenadines"),
    ("WS", "685", "Samoa"),
    ("SM", "378", "San Marino"),
    ("ST", "239", "São Tomé and Príncipe"),
    ("SA", "966", "Saudi Arabia"),
    ("SN", "221", "Senegal"),
    ("RS", "381", "Serbia"),
    ("SC", "248", "Seychelles"),
    ("SL", "232", "Sierra Leone"),
    ("SG", "65", "Singapore"),
    ("SK", "421", "Slovakia"),
    ("SI", "386", "Slovenia"),
    ("SB", "677", "Solomon Islands"),
    ("SO", "252", "Somalia"),
    ("ZA", "27", "South Africa"),
    ("KR", "82", "South Korea"),
    ("SS", "211", "South Sudan"),
    ("ES", "34", "Spain"),
    ("LK", "94", "Sri Lanka"),
    ("SD", "249", "Sudan"),
    ("SR", "597", "Suriname"),
    ("SE", "46", "Sweden"),
    ("CH", "41", "Switzerland"),
    ("SY", "963", "Syria"),
    ("TW", "886", "Taiwan"),
    ("TJ", "992", "Tajikistan"),
    ("TZ", "255", "Tanzania"),
    ("TH", "66", "Thailand"),
    ("TL", "670", "Timor-Leste"),
    ("TG", "228", "Togo"),
    ("TO", "676", "Tonga"),
    ("TT", "1", "Trinidad and Tobago"),
    ("TN", "216", "Tunisia"),
    ("TR", "90", "Turkey"),
    ("TM", "993", "Turkmenistan"),
    ("TV", "688", "Tuvalu"),
    ("UG", "256", "Uganda"),
    ("AE", "971", "United Arab Emirates"),
    ("UY", "598", "Uruguay"),
    ("UZ", "998", "Uzbekistan"),
    ("VU", "678", "Vanuatu"),
    ("VA", "39", "Vatican City"),
    ("VE", "58", "Venezuela"),
    ("VN", "84", "Vietnam"),
    ("YE", "967", "Yemen"),
    ("ZM", "260", "Zambia"),
    ("ZW", "263", "Zimbabwe"),
];

/// Flag emoji for an ISO country code ("IL" → 🇮🇱).
pub fn flag(iso: &str) -> String {
    iso.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| char::from_u32(0x1F1E6 + (c.to_ascii_uppercase() as u32 - 'A' as u32)).unwrap_or(' '))
        .collect()
}

pub fn country(iso: &str) -> Option<(&'static str, &'static str, &'static str)> {
    COUNTRIES.iter().copied().find(|(c, _, _)| c.eq_ignore_ascii_case(iso))
}

/// Builds a full international number ("+972501234567") from a country and
/// what the person typed. A leading 0 of the national number is dropped;
/// a number typed with "+" is taken as already international.
pub fn normalize(iso: &str, typed: &str) -> Result<String, &'static str> {
    let (_, dial, _) = country(iso).ok_or("Choose your country")?;
    let trimmed = typed.trim();
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if trimmed.chars().any(|c| !(c.is_ascii_digit() || " +-().".contains(c))) {
        return Err("The phone number may contain only digits");
    }
    let full = if trimmed.starts_with('+') || trimmed.starts_with("00") {
        digits.trim_start_matches("00").to_string()
    } else {
        format!("{dial}{}", digits.trim_start_matches('0'))
    };
    let national_len = full.len().saturating_sub(dial.len());
    if !full.starts_with(dial) || !(4..=14).contains(&national_len) || full.len() > 15 {
        return Err("Please enter a valid phone number");
    }
    Ok(format!("+{full}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_numbers() {
        assert_eq!(normalize("IL", "050-123-4567"), Ok("+972501234567".into()));
        assert_eq!(normalize("IL", "+972 50 123 4567"), Ok("+972501234567".into()));
        assert_eq!(normalize("US", "(415) 555-0123"), Ok("+14155550123".into()));
        assert_eq!(normalize("GB", "07700 900123"), Ok("+447700900123".into()));
        assert!(normalize("IL", "+1 415 555 0123").is_err(), "number from another country");
        assert!(normalize("IL", "12").is_err());
        assert!(normalize("IL", "abc").is_err());
        assert!(normalize("XX", "0501234567").is_err());
        assert_eq!(flag("il"), "🇮🇱");
        let mut seen = std::collections::HashSet::new();
        for (c, d, _) in COUNTRIES {
            assert!(seen.insert(*c), "duplicate {c}");
            assert!(d.chars().all(|ch| ch.is_ascii_digit()));
        }
    }
}
