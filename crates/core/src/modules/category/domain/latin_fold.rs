/// The text without the accents of Latin letters: «Migración» gives
/// «Migracion», «Ñandú» gives «Nandu». It works both on letters written
/// with their accent (NFC) and on a letter followed by its accent (NFD, as
/// macOS gives file names). Everything else stays as it was.
pub fn fold_latin(text: &str) -> String {
    text.chars()
        .filter(|c| !is_combining_accent(*c))
        .map(plain_letter)
        .collect()
}

/// The accents that NFD writes after their letter (Combining Diacritical
/// Marks).
fn is_combining_accent(c: char) -> bool {
    ('\u{300}'..='\u{36f}').contains(&c)
}

fn plain_letter(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'Á' | 'À' | 'Ä' | 'Â' | 'Ã' | 'Å' | 'Ā' | 'Ă' | 'Ą' => 'A',
        'é' | 'è' | 'ë' | 'ê' | 'ē' | 'ė' | 'ę' | 'ě' => 'e',
        'É' | 'È' | 'Ë' | 'Ê' | 'Ē' | 'Ė' | 'Ę' | 'Ě' => 'E',
        'í' | 'ì' | 'ï' | 'î' | 'ī' | 'į' => 'i',
        'Í' | 'Ì' | 'Ï' | 'Î' | 'Ī' | 'Į' => 'I',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => 'o',
        'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' | 'Ø' | 'Ō' | 'Ő' => 'O',
        'ú' | 'ù' | 'ü' | 'û' | 'ū' | 'ů' | 'ű' | 'ų' => 'u',
        'Ú' | 'Ù' | 'Ü' | 'Û' | 'Ū' | 'Ů' | 'Ű' | 'Ų' => 'U',
        'ñ' | 'ń' | 'ň' => 'n',
        'Ñ' | 'Ń' | 'Ň' => 'N',
        'ç' | 'ć' | 'č' => 'c',
        'Ç' | 'Ć' | 'Č' => 'C',
        'ý' | 'ÿ' => 'y',
        'Ý' | 'Ÿ' => 'Y',
        'š' | 'ś' => 's',
        'Š' | 'Ś' => 'S',
        'ž' | 'ź' | 'ż' => 'z',
        'Ž' | 'Ź' | 'Ż' => 'Z',
        'ł' => 'l',
        'Ł' => 'L',
        'ř' => 'r',
        'Ř' => 'R',
        'ğ' => 'g',
        'Ğ' => 'G',
        other => other,
    }
}
