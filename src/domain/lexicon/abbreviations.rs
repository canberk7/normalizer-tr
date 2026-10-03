//! Every abbreviation the lexicon reads, in one table of how each is written and said.

use std::{collections::HashMap, sync::OnceLock};

use super::{Lexeme, lookup_key};
use crate::morphology::{Harmony, Word, WordEnd, vowel_harmony};

/// One abbreviation: its written form, what is said for it and what its suffix follows.
#[derive(Clone, Copy)]
struct Abbreviation {
    written: &'static str,
    said: &'static str,
    /// The letter name a suffix is written for when the abbreviation is said in full:
    /// `KDV'ye` follows "ve", though "katma değer vergisi" is said.
    letter: Option<&'static str>,
    /// The ending of the last word said, where its letters alone do not show it.
    end: Option<WordEnd>,
    /// Whether the all-capitals spelling reads it too: `DR.` as `Dr.`.
    capitals: bool,
    /// The form said after an ordinal number, where a name would take the possessive:
    /// `123. Sok.` is yüz yirmi üçüncü sokak, `Gül Sok.` Gül sokağı.
    bare: Option<&'static str>,
    /// The possessive form said after a noun it makes a compound with: `Sipariş No:` is
    /// sipariş numarası, `No: 5` numara beş.
    compound: Option<&'static str>,
    /// Where it is read at all.
    context: Context,
}

/// Where an abbreviation is read. One that could be something else, an ordinary word above
/// all (`Bul.` is "find!", `Kas.` "muscle"), is read only where nothing else fits; elsewhere
/// it stays as written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Context {
    /// Wherever it stands: it means nothing else.
    Free,
    /// Right before a capitalized word, as a title before a name: `Av. Ayşe`.
    Title,
    /// Right after a name, an abbreviation or a number, as a place: `Atatürk Bul.`,
    /// `Müh. Böl.`, `1234. Sok.`.
    Place,
    /// Right before a number: `s. 12`, `min. 18`, `No 5`.
    BeforeNumber,
    /// Next to a number, as a date or a time is: `5 Haz.`, `Haz. 2024`, `9 AM`.
    Dated,
    /// Before a lowercase word or a number, which shows the sentence goes on after its
    /// period: `ör. bir`, `Sal. günü`.
    Continues,
    /// Anywhere but right before a number, which no unit is written before: `MB cinsinden`,
    /// but not the `GB` of `GB 18030`.
    Unit,
    /// Anywhere but right after a number, which makes it a unit: `min. tutar` is minimum,
    /// `30 min.` minutes.
    Uncounted,
}

/// An abbreviation said as `said`, whose suffix follows the last word said.
const fn said(written: &'static str, said: &'static str) -> Abbreviation {
    Abbreviation {
        written,
        said,
        letter: None,
        end: None,
        capitals: true,
        bare: None,
        compound: None,
        context: Context::Free,
    }
}

impl Abbreviation {
    /// A suffix follows this letter name, as it is written for the abbreviation's letters.
    const fn written_for(self, letter: &'static str) -> Self {
        Self {
            letter: Some(letter),
            ..self
        }
    }
    /// The last word said ends in a compound's possessive: `caddesi`, `vergisi`.
    const fn possessive(self) -> Self {
        Self {
            end: Some(WordEnd::Possessive),
            ..self
        }
    }
    /// The last word said softens its final `t` before a vowel: `cilt`, `cildi`.
    const fn softens(self) -> Self {
        Self {
            end: Some(WordEnd::Softens),
            ..self
        }
    }
    /// The last word said softens its final `k` before a vowel: `blok`, `bloğu`.
    const fn softens_k(self) -> Self {
        Self {
            end: Some(WordEnd::SoftensK),
            ..self
        }
    }
    /// Only this spelling reads it, since in capitals it is a common word: `BUL.` is "find".
    const fn exact_case(self) -> Self {
        Self {
            capitals: false,
            ..self
        }
    }
    /// The form said after an ordinal number, without the possessive of a name.
    const fn bare(self, bare: &'static str) -> Self {
        Self {
            bare: Some(bare),
            ..self
        }
    }
    /// The possessive form said after a noun, which ends a compound with it.
    const fn compound(self, compound: &'static str) -> Self {
        Self {
            compound: Some(compound),
            ..self
        }
    }
    /// A title whose letters are a word, read only before a name, in its own spelling.
    const fn title(self) -> Self {
        self.only(Context::Title)
    }
    /// A place whose letters are a word, read only after a name or a number.
    const fn place(self) -> Self {
        self.only(Context::Place)
    }
    /// Read only before a number.
    const fn before_number(self) -> Self {
        self.only(Context::BeforeNumber)
    }
    /// A month, a day or a time of day, read only next to a number.
    const fn dated(self) -> Self {
        self.only(Context::Dated)
    }
    /// Read only before a lowercase word or a number.
    const fn continues(self) -> Self {
        self.only(Context::Continues)
    }
    /// A unit, read anywhere but before a number, in its own case only.
    const fn unit(self) -> Self {
        self.only(Context::Unit)
    }
    /// Read anywhere but after a number, where it is a unit.
    const fn uncounted(self) -> Self {
        self.only(Context::Uncounted)
    }
    /// Read only in `context`, and only in this spelling: in capitals the letters of one that
    /// could be something else are more often that.
    const fn only(self, context: Context) -> Self {
        Self {
            context,
            capitals: false,
            ..self
        }
    }
    fn lexeme(self) -> Lexeme {
        self.lexeme_of(self.said, self.end)
    }
    fn lexeme_of(self, said: &'static str, end: Option<WordEnd>) -> Lexeme {
        let last = said.rsplit(' ').next().unwrap_or(said);
        let target = said_word(last, end);
        let source = self.letter.map_or(target, |letter| said_word(letter, None));
        Lexeme::distinct(said, source, target)
    }
    fn reading(self) -> Reading {
        Reading {
            lexeme: self.lexeme(),
            bare: self.bare.map(|bare| self.lexeme_of(bare, None)),
            compound: self
                .compound
                .map(|compound| self.lexeme_of(compound, Some(WordEnd::Possessive))),
            context: self.context,
        }
    }
    /// The all-capitals spelling that reads a dotted or mixed-case abbreviation too, for one
    /// of two letters or more that is not already written so.
    fn capitals_key(self) -> Option<String> {
        let key = capitals(self.written);
        let letters = self.written.chars().filter(|ch| ch.is_alphabetic()).count();
        (self.capitals && letters > 1 && key != self.written).then_some(key)
    }
}

/// Every abbreviation the lexicon reads. An initialism is said as people say it, mostly
/// letter by letter (`ABD` a be de) and some as a word (`NATO`, `ODTÜ`); a dotted
/// abbreviation is said in full, in the possessive form it takes at the end of a name
/// (`Cumhuriyet Cad.` is Cumhuriyet caddesi).
const ABBREVIATIONS: &[Abbreviation] = &[
    // The approved entries this table grew from, read as before.
    said("Dr.", "doktor"),
    said("Prof.", "profesör"),
    said("vb.", "ve benzeri").written_for("be"),
    said("TBMM", "te be me me"),
    said("PTT", "pe te te"),
    said("NATO", "nato"),
    said("IBAN", "iban"),
    said("KDV", "katma değer vergisi")
        .written_for("ve")
        .possessive(),
    // Academic, professional and honorific titles.
    said("Doç.", "doçent"),
    said("Yrd.", "yardımcı"),
    said("Yard.", "yardımcı"),
    said("Ord.", "ordinaryüs"),
    said("Öğr.", "öğretim"),
    said("Öğrt.", "öğretmen"),
    // Titles whose letters are also a word (`gör`, `av`) are read only before a name.
    said("Gör.", "görevlisi").possessive().title(),
    said("Arş.", "araştırma"),
    said("Okt.", "okutman"),
    said("Uzm.", "uzman"),
    said("Av.", "avukat").title(),
    said("Müh.", "mühendis"),
    said("Mim.", "mimar").title(),
    said("Ecz.", "eczacı"),
    said("Dt.", "diş hekimi").possessive(),
    said("Dyt.", "diyetisyen"),
    said("Psk.", "psikolog"),
    said("Op.", "operatör"),
    said("Opr.", "operatör"),
    said("Hem.", "hemşire").title(),
    said("Vet.", "veteriner"),
    // Titles written together with no space, as they often are.
    said("Prof.Dr.", "profesör doktor"),
    said("Doç.Dr.", "doçent doktor"),
    said("Yrd.Doç.Dr.", "yardımcı doçent doktor"),
    said("Dr.Öğr.Üyesi", "doktor öğretim üyesi").possessive(),
    said("Op.Dr.", "operatör doktor"),
    said("Uzm.Dr.", "uzman doktor"),
    said("Arş.Gör.", "araştırma görevlisi").possessive(),
    said("Öğr.Gör.", "öğretim görevlisi").possessive(),
    said("Arş.Gör.Dr.", "araştırma görevlisi doktor"),
    said("Öğr.Gör.Dr.", "öğretim görevlisi doktor"),
    said("Fzt.", "fizyoterapist"),
    said("Sn.", "sayın"),
    said("Hz.", "hazreti"),
    said("Bşk.", "başkanı").possessive(),
    said("Gn.", "genel"),
    said("Gnl.", "genel"),
    said("Md.", "müdürlüğü").possessive(),
    said("Müd.", "müdürlüğü").possessive(),
    said("Sek.", "sekreter").title(),
    said("Şb.", "şubesi").possessive().bare("şube"),
    // Military ranks and services.
    said("Org.", "orgeneral"),
    said("Korg.", "korgeneral"),
    said("Tümg.", "tümgeneral"),
    said("Tuğg.", "tuğgeneral"),
    said("Gen.", "general").title(),
    said("Alb.", "albay"),
    said("Yb.", "yarbay"),
    said("Bnb.", "binbaşı"),
    said("Yzb.", "yüzbaşı"),
    said("Ütğm.", "üsteğmen"),
    said("Tğm.", "teğmen"),
    said("Asb.", "astsubay"),
    said("Kd.", "kıdemli"),
    said("Çvş.", "çavuş"),
    said("Bçvş.", "başçavuş"),
    said("Onb.", "onbaşı"),
    said("Amr.", "amiral").title(),
    said("Ora.", "oramiral").title(),
    said("Kora.", "koramiral"),
    said("Tüma.", "tümamiral"),
    said("Tuğa.", "tuğamiral"),
    said("Gnkur.", "genelkurmay"),
    said("Hv.", "hava"),
    said("Dz.", "deniz"),
    // Addresses. A place whose letters are also a word (`sok`, `bul`) is read only after the
    // name or the number of the place.
    said("Cad.", "caddesi").possessive().bare("cadde"),
    said("Cd.", "caddesi").possessive().bare("cadde"),
    said("Sok.", "sokağı").possessive().place().bare("sokak"),
    said("Sk.", "sokağı").possessive().bare("sokak"),
    said("Mah.", "mahallesi").possessive().bare("mahalle"),
    said("Mh.", "mahallesi").possessive().bare("mahalle"),
    said("Bul.", "bulvarı").possessive().place().bare("bulvar"),
    said("Blv.", "bulvarı").possessive().bare("bulvar"),
    said("Apt.", "apartmanı").possessive().bare("apartman"),
    said("Ap.", "apartmanı").possessive().bare("apartman"),
    said("Sit.", "sitesi").possessive().place().bare("site"),
    said("Mevk.", "mevkii").possessive().bare("mevki"),
    said("Mvk.", "mevkii").possessive().bare("mevki"),
    said("Mrk.", "merkezi").possessive().bare("merkez"),
    said("Plz.", "plaza"),
    said("Bl.", "blok").softens_k(),
    said("Blk.", "blok").softens_k(),
    said("İlç.", "ilçesi").possessive().bare("ilçe"),
    // Institutions, said in the possessive form a name takes (`Ankara Üniv.`).
    said("Üniv.", "üniversitesi")
        .possessive()
        .bare("üniversite"),
    said("Ünv.", "üniversitesi").possessive().bare("üniversite"),
    said("Fak.", "fakültesi").possessive().bare("fakülte"),
    said("Ens.", "enstitüsü").possessive().bare("enstitü"),
    said("Böl.", "bölümü").possessive().place().bare("bölüm"),
    said("Hst.", "hastanesi").possessive().bare("hastane"),
    said("Hast.", "hastanesi").possessive().bare("hastane"),
    said("Lis.", "lisesi").possessive().bare("lise"),
    said("Okl.", "okulu").possessive().bare("okul"),
    said("Der.", "derneği").possessive().place(),
    said("Vak.", "vakfı").possessive(),
    said("Koop.", "kooperatifi").possessive(),
    said("No.", "numara").compound("numarası"),
    said("No:", "numara").compound("numarası"),
    said("no.", "numara").compound("numarası"),
    said("no:", "numara").compound("numarası"),
    said("Nu.", "numara").compound("numarası"),
    said("Nr.", "numara").compound("numarası"),
    // Without a period only a number after it makes it one: `Kapı No 5`, but `No, I can't`.
    said("No", "numara").compound("numarası").before_number(),
    said("NO", "numara").compound("numarası").before_number(),
    said("D:", "daire").before_number(),
    said("Tel.", "telefon").before_number(),
    said("Tel:", "telefon"),
    said("P.K.", "posta kutusu").possessive(),
    // Companies and trades.
    said("Ltd.", "limited"),
    said("Şti.", "şirketi").possessive(),
    said("A.Ş.", "anonim şirketi").possessive(),
    said("AŞ", "anonim şirketi").written_for("şe").possessive(),
    said("Koll.", "kolektif"),
    said("Tic.", "ticaret"),
    said("San.", "sanayi").place(),
    said("İth.", "ithalat"),
    said("İhr.", "ihracat"),
    said("İnş.", "inşaat"),
    said("Taah.", "taahhüt").softens(),
    said("Mak.", "makine"),
    said("Elk.", "elektrik").softens_k(),
    said("Nak.", "nakliyat"),
    said("Teks.", "tekstil"),
    said("Hiz.", "hizmetleri").possessive(),
    said("Yay.", "yayınları").possessive().place(),
    said("Ar-Ge", "araştırma geliştirme"),
    said("Ür-Ge", "ürün geliştirme"),
    // Prose, references and law. `vs.` is not read: it is vesaire after a list and versus
    // between two names, so its letters are said. A single letter is one only before a number:
    // `s. 12`, but not the answer `c.` or the item `d.` of a list.
    said("vd.", "ve diğerleri").written_for("de"),
    said("bkz.", "bakınız"),
    said("bk.", "bakınız"),
    said("örn.", "örneğin"),
    said("ör.", "örnek").softens_k().continues(),
    said("yy.", "yüzyıl"),
    said("s.", "sayfa").before_number(),
    said("ss.", "sayfalar").before_number(),
    said("c.", "cilt").softens().before_number(),
    said("çev.", "çeviren"),
    said("ed.", "editör"),
    said("krş.", "karşılaştırınız"),
    said("a.g.e.", "adı geçen eser"),
    said("a.g.m.", "adı geçen makale"),
    said("ç.n.", "çevirenin notu").possessive(),
    said("d.", "doğum").before_number(),
    said("ö.", "ölüm").before_number(),
    said("M.Ö.", "milattan önce"),
    said("MÖ", "milattan önce").written_for("ö"),
    said("M.S.", "milattan sonra"),
    said("s.a.v.", "sallallahu aleyhi ve sellem"),
    said("a.s.", "aleyhisselam"),
    said("r.a.", "radıyallahu anh"),
    said("md.", "madde"),
    said("mad.", "madde"),
    said("fık.", "fıkra"),
    said("bnd.", "bent").softens(),
    said("maks.", "maksimum"),
    said("max.", "maksimum"),
    // After a number it is minutes: `min. 18 yaş` is minimum, `30 min.` stays.
    said("min.", "minimum").uncounted(),
    said("ort.", "ortalama"),
    said("yakl.", "yaklaşık"),
    said("takr.", "takriben"),
    said("T.C.", "Türkiye Cumhuriyeti").possessive(),
    said("TC", "te ce"),
    said("RG", "Resmi Gazete").written_for("ge"),
    // Days and months. A day whose letters are a word (`sal`, `çar`) is read only where the
    // sentence goes on after it (`Sal. günü`), and a month only next to a number (`5 Kas.`).
    said("Pzt.", "pazartesi"),
    said("Sal.", "salı").continues(),
    said("Çrş.", "çarşamba"),
    said("Çar.", "çarşamba").continues(),
    said("Prş.", "perşembe"),
    said("Per.", "perşembe").continues(),
    said("Cum.", "cuma"),
    said("Cmt.", "cumartesi"),
    said("Cts.", "cumartesi"),
    said("Pzr.", "pazar"),
    said("Oca.", "Ocak").softens_k().dated(),
    said("Şub.", "Şubat").dated(),
    said("Mar.", "Mart").dated(),
    said("Nis.", "Nisan").dated(),
    said("May.", "Mayıs").dated(),
    said("Haz.", "Haziran").dated(),
    said("Tem.", "Temmuz").dated(),
    said("Ağu.", "Ağustos").dated(),
    said("Ağus.", "Ağustos").dated(),
    said("Eyl.", "Eylül").dated(),
    said("Eki.", "Ekim").dated(),
    said("Kas.", "Kasım").dated(),
    // Languages, but not those whose letters are a word or a people (`Ar.`, `Rus.`).
    said("Alm.", "Almanca"),
    said("Fr.", "Fransızca"),
    said("İng.", "İngilizce"),
    said("İsp.", "İspanyolca"),
    said("Lat.", "Latince"),
    said("Yun.", "Yunanca"),
    said("Jap.", "Japonca"),
    // Initialisms said letter by letter.
    said("AA", "a a"),
    said("AB", "a be"),
    said("ABD", "a be de"),
    said("AİHM", "a i he me"),
    said("AKM", "a ke me"),
    said("AKP", "a ke pe"),
    said("ATM", "a te me"),
    said("ATV", "a te ve"),
    said("AVM", "a ve me"),
    said("AYM", "a ye me"),
    said("BBP", "be be pe"),
    said("BDDK", "be de de ke"),
    said("BJK", "be je ke"),
    said("BM", "be me"),
    said("BSMV", "be se me ve"),
    said("BT", "be te"),
    said("BTK", "be te ke"),
    said("CHP", "ce he pe"),
    said("CMK", "ce me ke"),
    said("DHMİ", "de he me i"),
    said("DİB", "de i be"),
    said("DNA", "de ne a"),
    said("DSİ", "de se i"),
    said("DSÖ", "de se ö"),
    said("DSP", "de se pe"),
    said("DVD", "de ve de"),
    said("EEG", "e e ge"),
    said("EFT", "e fe te"),
    said("EGM", "e ge me"),
    said("EKG", "e ke ge"),
    said("EYT", "e ye te"),
    said("GPS", "ge pe se"),
    said("GSM", "ge se me"),
    said("GSYH", "ge se ye he"),
    said("HD", "he de"),
    said("HDP", "he de pe"),
    said("HIV", "he i ve"),
    said("HMK", "he me ke"),
    said("HSK", "he se ke"),
    said("İBB", "i be be"),
    said("İETT", "i e te te"),
    said("İİK", "i i ke"),
    said("İTO", "i te o"),
    said("İTÜ", "i te ü"),
    said("KGM", "ke ge me"),
    said("KKDF", "ke ke de fe"),
    said("KKTC", "ke ke te ce"),
    said("KPSS", "ke pe se se"),
    said("KTÜ", "ke te ü"),
    said("KVKK", "ke ve ke ke"),
    said("LCD", "le ce de"),
    said("LGS", "le ge se"),
    said("MHP", "me he pe"),
    said("MHRS", "me he re se"),
    said("MKE", "me ke e"),
    said("MSB", "me se be"),
    said("MSÜ", "me se ü"),
    said("MTA", "me te a"),
    said("NTV", "ne te ve"),
    said("OSB", "o se be"),
    said("ÖSYM", "ö se ye me"),
    said("ÖTV", "ö te ve"),
    said("PCR", "pe ce re"),
    said("PDF", "pe de fe"),
    said("PKK", "pe ke ke"),
    said("RNA", "re ne a"),
    said("SGK", "se ge ke"),
    said("SMS", "se me se"),
    said("SPK", "se pe ke"),
    said("SSK", "se se ke"),
    said("SSS", "se se se"),
    said("TCDD", "te ce de de"),
    said("TCK", "te ce ke"),
    said("TCMB", "te ce me be"),
    said("TDK", "te de ke"),
    said("TEB", "te e be"),
    said("TFF", "te fe fe"),
    said("THY", "te he ye"),
    said("TMK", "te me ke"),
    said("TMMOB", "te me me o be"),
    said("TMO", "te me o"),
    said("TMSF", "te me se fe"),
    said("TOBB", "te o be be"),
    said("TPAO", "te pe a o"),
    said("TRT", "te re te"),
    said("TSE", "te se e"),
    said("TSK", "te se ke"),
    said("TTB", "te te be"),
    said("TV", "te ve"),
    said("USB", "u se be"),
    said("YDS", "ye de se"),
    said("YKS", "ye ke se"),
    said("YSK", "ye se ke"),
    said("YTÜ", "ye te ü"),
    said("API", "a pe i"),
    said("BTU", "be te u"),
    said("CPU", "ce pe u"),
    said("GPU", "ge pe u"),
    said("HDMI", "he de me i"),
    said("IP", "i pe"),
    said("LTE", "le te e"),
    said("NVMe", "ne ve me e").exact_case(),
    said("SKU", "se ke u"),
    said("SpO", "se pe o").exact_case(),
    said("UHD", "u he de"),
    said("URL", "u re le"),
    said("UV", "u ve"),
    // Initialisms said with the English letter names they are known by.
    said("BBC", "bi bi si"),
    said("BMW", "be em ve"),
    said("CD", "si di"),
    said("CEO", "si i o"),
    said("CIA", "si ay ey"),
    said("CNN", "si en en"),
    said("CV", "si vi"),
    said("DHL", "de he le"),
    said("DJ", "di cey"),
    said("FBI", "ef bi ay"),
    said("HSBC", "he se be ce"),
    said("IBM", "ay bi em"),
    said("IMF", "ay em ef"),
    said("KFC", "ke ef si"),
    said("KGB", "ke ge be"),
    said("LG", "el ci"),
    said("MR", "em ar"),
    said("NBA", "en bi ey"),
    said("PC", "pi si"),
    said("PR", "pi ar"),
    said("VR", "vi ar"),
    said("WiFi", "vay fay").exact_case(),
    said("Wi-Fi", "vay fay"),
    said("iOS", "ay o es").exact_case(),
    said("OK", "okey"),
    // The time of day only next to a time: `9 AM`, but `I AM OK` is English.
    said("PM", "pi em").dated(),
    said("AM", "ey em").dated(),
    said("pm", "pi em").dated(),
    said("am", "ey em").dated(),
    said("ÖÖ", "öğleden önce").dated(),
    said("ÖS", "öğleden sonra").dated(),
    said("Ö.Ö.", "öğleden önce").dated(),
    said("Ö.S.", "öğleden sonra").dated(),
    said("MAC", "mek"),
    // Initialisms said as a word.
    said("AFAD", "afad"),
    said("AGİT", "agit"),
    said("AKUT", "akut"),
    said("ALES", "ales"),
    said("ASELSAN", "aselsan"),
    said("ASKİ", "aski"),
    said("BAĞ-KUR", "bağkur"),
    said("Bağ-Kur", "bağkur"),
    said("BİM", "bim"),
    said("BİST", "bist"),
    said("BIST", "bist"),
    said("BOTAŞ", "botaş"),
    said("COVID", "kovid"),
    said("DEAŞ", "deaş"),
    said("DİSK", "disk"),
    said("FETÖ", "fetö"),
    said("FIFA", "fifa"),
    said("HAK-İŞ", "hak iş"),
    said("HAVELSAN", "havelsan"),
    said("IŞİD", "ışid"),
    said("İGDAŞ", "igdaş"),
    said("İHA", "iha"),
    said("İSKİ", "iski"),
    said("İŞKUR", "işkur"),
    said("KESK", "kesk"),
    said("KOAH", "koah"),
    said("KOBİ", "kobi"),
    said("KOSGEB", "kosgeb"),
    said("LED", "led"),
    said("MEB", "meb"),
    said("MİT", "mit"),
    said("MÜSİAD", "müsiad"),
    said("NASA", "nasa"),
    said("ODTÜ", "odtü"),
    said("OPEC", "opek"),
    said("PIN", "pin"),
    said("SIM", "sim"),
    said("ISO", "iso"),
    said("GIF", "gif"),
    said("POS", "pos"),
    said("ROKETSAN", "roketsan"),
    said("SİHA", "siha"),
    said("SWIFT", "svift"),
    said("TEDAŞ", "tedaş"),
    said("TEİAŞ", "teiaş"),
    said("TİGEM", "tigem"),
    said("TİKA", "tika"),
    said("TOGG", "tog"),
    said("TOKİ", "toki"),
    said("TUSAŞ", "tusaş"),
    said("TÜBİTAK", "tübitak"),
    said("TÜFE", "tüfe"),
    said("TÜİK", "tüik"),
    said("TÜPRAŞ", "tüpraş"),
    said("TÜRK-İŞ", "türk iş"),
    said("TÜSİAD", "tüsiad"),
    said("UEFA", "uefa"),
    said("UNESCO", "yunesko"),
    said("UNICEF", "yunisef"),
    said("ÜFE", "üfe"),
    said("VIP", "vip"),
    said("YÖK", "yök"),
    // Measures said wherever they stand, but in their own case only (`MAH` is no `mAh`) and
    // not right before a number, which no unit is written before.
    said("KB", "kilobayt").written_for("be").unit(),
    said("MB", "megabayt").written_for("be").unit(),
    said("GB", "gigabayt").written_for("be").unit(),
    said("TB", "terabayt").written_for("be").unit(),
    said("GHz", "gigahertz").unit(),
    said("MHz", "megahertz").unit(),
    said("kHz", "kilohertz").unit(),
    said("kW", "kilovat").unit(),
    said("kWh", "kilovatsaat").unit(),
    said("mAh", "miliamper saat").unit(),
    said("dB", "desibel").unit(),
    said("Mbps", "megabit").unit(),
    said("Gbps", "gigabit").unit(),
    said("MMBtu", "milyon be te u").unit(),
    said("mmHg", "milimetre cıva").unit(),
    said("pH", "pe he").exact_case(),
    said("dL", "desilitre").unit(),
    said("dl", "desilitre").unit(),
    said("IU", "ünite").unit(),
    said("dpi", "de pe i").unit(),
    said("DPI", "de pe i").unit(),
    said("ppi", "pe pe i").unit(),
    said("PPI", "pe pe i").unit(),
    said("fps", "fe pe se").unit(),
    said("FPS", "fe pe se").unit(),
    said("rpm", "re pe me").unit(),
    said("RPM", "re pe me").unit(),
    said("SARS-CoV-2", "sars kov iki"),
    // Chat abbreviations, said as the words they stand for. Only these lowercase spellings
    // read them: no Turkish word is written without its vowels.
    said("slm", "selam").exact_case(),
    said("slmlr", "selamlar").exact_case(),
    said("mrb", "merhaba").exact_case(),
    said("mrhb", "merhaba").exact_case(),
    said("mrblr", "merhabalar").exact_case(),
    said("nbr", "naber").exact_case(),
    said("nsl", "nasıl").exact_case(),
    said("nslsn", "nasılsın").exact_case(),
    said("tmm", "tamam").exact_case(),
    said("tmmdr", "tamamdır").exact_case(),
    said("tşk", "teşekkürler").exact_case(),
    said("tşkr", "teşekkürler").exact_case(),
    said("tşkler", "teşekkürler").exact_case(),
    said("tşkrler", "teşekkürler").exact_case(),
    said("tsk", "teşekkürler").exact_case(),
    said("sğl", "sağ ol").exact_case(),
    said("sğol", "sağ ol").exact_case(),
    said("yrn", "yarın").exact_case(),
    said("knk", "kanka").exact_case(),
    said("cnm", "canım").exact_case(),
    said("kib", "kendine iyi bak").exact_case(),
    said("grşz", "görüşürüz").exact_case(),
    said("grşrz", "görüşürüz").exact_case(),
    said("hrks", "herkes").exact_case(),
    said("bnm", "benim").exact_case(),
    said("snn", "senin").exact_case(),
    said("nrd", "nerede").exact_case(),
    said("nrde", "nerede").exact_case(),
    said("bknz", "bakınız").exact_case(),
    said("bkz", "bakınız").exact_case(),
    said("vb", "ve benzeri").written_for("be").exact_case(),
    said("msj", "mesaj").exact_case(),
    said("tlf", "telefon").exact_case(),
    said("pls", "plis").exact_case(),
    said("plz", "plis").exact_case(),
    said("zmn", "zaman").exact_case(),
    said("bn", "ben").exact_case(),
    said("cvp", "cevap").exact_case(),
    said("kdşm", "kardeşim").exact_case(),
    said("krdşm", "kardeşim").exact_case(),
    said("hcm", "hocam").exact_case(),
    said("bşy", "bir şey").exact_case(),
    said("pzt", "pazartesi").exact_case(),
    said("cmt", "cumartesi").exact_case(),
    // Greetings written with a point between their letters.
    said("s.a", "selamünaleyküm").exact_case(),
    said("s.a.", "selamünaleyküm").exact_case(),
    said("a.s", "aleykümselam").exact_case(),
];

/// The table looked up by written form, and by the all-capitals form where that reads too.
struct Table {
    forms: HashMap<String, Reading>,
    /// The few forms that are lowercase words without a period (`kW`, `dB`), searched one
    /// by one: most words are such, and almost none of them is an abbreviation.
    lowercase: Vec<(&'static str, Reading)>,
}

/// What an abbreviation is said as, after an ordinal number or a noun where that differs, and
/// where it is read at all.
#[derive(Clone, Copy)]
struct Reading {
    lexeme: Lexeme,
    bare: Option<Lexeme>,
    compound: Option<Lexeme>,
    context: Context,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut forms = HashMap::with_capacity(ABBREVIATIONS.len() * 2);
        let mut lowercase = Vec::new();
        for entry in ABBREVIATIONS {
            let reading = entry.reading();
            forms.insert(entry.written.to_owned(), reading);
            if lowercase_word(entry.written) {
                lowercase.push((entry.written, reading));
            }
        }
        // A written form keeps its own reading, and the first entry with a capitals form
        // keeps that: `Md.` müdürlüğü before `md.`.
        for entry in ABBREVIATIONS {
            if let Some(key) = entry.capitals_key() {
                forms.entry(key).or_insert_with(|| entry.reading());
            }
        }
        Table { forms, lowercase }
    })
}

fn find(symbol: &str) -> Option<Reading> {
    let table = table();
    if lowercase_word(symbol) {
        return table
            .lowercase
            .iter()
            .find(|(written, _)| *written == symbol)
            .map(|(_, reading)| *reading);
    }
    table.forms.get(symbol).copied()
}

/// A lowercase word without a period, which is what most words in a text are.
fn lowercase_word(text: &str) -> bool {
    text.starts_with(char::is_lowercase) && !text.contains('.')
}

/// Turkish capitals, in which `i` is `İ` and `ı` is `I`.
fn capitals(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            'i' => 'İ'.to_uppercase(),
            'ı' => 'I'.to_uppercase(),
            _ => ch.to_uppercase(),
        })
        .collect()
}

/// The suffix shape of a said word: the harmony of its last vowel, and its ending.
fn said_word(text: &'static str, end: Option<WordEnd>) -> Word {
    let key = lookup_key(text);
    let harmony = key
        .chars()
        .rev()
        .find_map(vowel_harmony)
        .unwrap_or(Harmony::FrontFlat);
    // A word of more than one syllable that ends in `k` softens it: sokak, sokağı.
    let syllables = key
        .chars()
        .filter(|letter| vowel_harmony(*letter).is_some())
        .count();
    let end = end.unwrap_or(match key.chars().last() {
        Some(letter) if vowel_harmony(letter).is_some() => WordEnd::Vowel,
        Some('k') if syllables > 1 => WordEnd::SoftensK,
        Some('f' | 's' | 't' | 'k' | 'ç' | 'ş' | 'h' | 'p') => WordEnd::Voiceless,
        _ => WordEnd::Voiced,
    });
    Word::new(text, harmony, end)
}

/// The words written right before and after an abbreviation, each separated from it by
/// whitespace only.
#[derive(Clone, Copy, Default)]
pub(crate) struct Around<'a> {
    pub(crate) previous: Option<&'a str>,
    pub(crate) next: Option<&'a str>,
}

/// What is written right before an abbreviation, where its reading depends on that.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Before {
    /// An ordinal number: `123. Sok.` is yüz yirmi üçüncü sokak.
    Ordinal,
    /// A noun the abbreviation makes a compound with: `Sipariş No:` is sipariş numarası.
    Noun,
    /// Nothing: a place said without a name before it is said bare, `Apt. veya Blok` is
    /// apartman veya blok.
    Nothing,
    /// Anything else.
    Other,
}

/// Words that name a street or a building, also abbreviated without a period. `No:` after one
/// is the number on it, not a compound with it: `İstiklal Caddesi No: 45` is İstiklal Caddesi
/// numara kırk beş, `İSTİKLAL CAD NO 5` İstiklal cad numara beş.
const PLACES: &[&str] = &[
    "cad",
    "cd",
    "sok",
    "sk",
    "mah",
    "mh",
    "bul",
    "blv",
    "apt",
    "ap",
    "sit",
    "bl",
    "blk",
    "cadde",
    "caddesi",
    "sokak",
    "sokağı",
    "mahalle",
    "mahallesi",
    "bulvar",
    "bulvarı",
    "meydan",
    "meydanı",
    "yol",
    "yolu",
    "çıkmaz",
    "çıkmazı",
    "apartman",
    "apartmanı",
    "site",
    "sitesi",
    "blok",
    "bloğu",
    "han",
    "hanı",
    "pasaj",
    "pasajı",
    "plaza",
    "plazası",
    "çarşı",
    "çarşısı",
];

/// Determiners, conjunctions and postpositions, which make no compound with the word after
/// them: `bir no.` is bir numara, `ve No: 5` ve numara beş.
const FUNCTION_WORDS: &[&str] = &[
    "bir", "bu", "şu", "o", "her", "hangi", "aynı", "yeni", "eski", "son", "ilk", "başka", "diğer",
    "öbür", "hiçbir", "birkaç", "bazı", "tüm", "bütün", "kendi", "ve", "veya", "ya", "da", "de",
    "ile", "ama", "için", "gibi", "kadar",
];

/// What a word written right before an abbreviation is to it: an ordinal number, or a noun
/// when it is all letters, no function word, and names no street or building.
fn before_word(previous: Option<&str>) -> Before {
    let Some(word) = previous else {
        return Before::Nothing;
    };
    if ordinal(word) {
        return Before::Ordinal;
    }
    let key = lookup_key(word);
    let noun = !word.is_empty()
        && word.chars().all(char::is_alphabetic)
        && !PLACES.contains(&key.as_str())
        && !FUNCTION_WORDS.contains(&key.as_str());
    if noun { Before::Noun } else { Before::Other }
}

/// A number written with an ordinal's period: `123.`.
fn ordinal(word: &str) -> bool {
    word.strip_suffix('.')
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether an abbreviation is read where it stands. A name before a place is a capitalized
/// word that ends no sentence, or an abbreviation itself: `Atatürk Bul.`, `Müh. Böl.`, but not
/// the `Topla.` of `Topla. Der.`
fn fits(context: Context, around: Around<'_>) -> bool {
    let previous = around.previous.unwrap_or_default();
    let next = around.next.unwrap_or_default();
    let number = |word: &str| word.starts_with(|ch: char| ch.is_ascii_digit());
    let named = previous.starts_with(char::is_uppercase)
        && !FUNCTION_WORDS.contains(&lookup_key(previous).as_str())
        && (!previous.ends_with(['.', '!', '?', ':', ';']) || abbreviation(previous).is_some());
    match context {
        Context::Free => true,
        Context::Title => next.starts_with(char::is_uppercase),
        Context::Place => named || number(previous),
        Context::BeforeNumber => number(next),
        Context::Dated => number(next) || previous.ends_with(|ch: char| ch.is_ascii_digit()),
        Context::Continues => next.starts_with(|ch: char| ch.is_lowercase() || ch.is_ascii_digit()),
        Context::Unit => !number(next),
        Context::Uncounted => !previous.ends_with(|ch: char| ch.is_ascii_digit()),
    }
}

/// Whether an abbreviation names a place numbered by an ordinal, as `Sok.` does: `1234. Sok.`
/// is bin iki yüz otuz dördüncü sokak.
pub(crate) fn numbered_place(symbol: &str) -> bool {
    find(symbol).is_some_and(|reading| reading.bare.is_some())
}

/// Whether the table has this spelling, wherever it would be read: the period of `Bul.` is
/// part of the word even where it ends a sentence.
pub(crate) fn written(symbol: &str) -> bool {
    find(symbol).is_some()
}

/// The reading of a unit the table spells this way, where it is read: after a number or with
/// no number after it (`5 GB`, `GB cinsinden`).
pub(crate) fn measure(symbol: &str) -> Option<Lexeme> {
    find(symbol)
        .filter(|reading| reading.context == Context::Unit)
        .map(|reading| reading.lexeme)
}

/// The reading of an abbreviation that means nothing else, as written or, dotted, in capitals.
pub(crate) fn abbreviation(symbol: &str) -> Option<Lexeme> {
    find(symbol)
        .filter(|reading| reading.context == Context::Free)
        .map(|reading| reading.lexeme)
}

/// The reading of an abbreviation between the words around it, where they let it be read:
/// without the possessive a name would take after an ordinal number or with no word before
/// it, and in the possessive of a compound after a noun.
pub(crate) fn abbreviation_around(symbol: &str, around: Around<'_>) -> Option<Lexeme> {
    let reading = find(symbol).filter(|reading| fits(reading.context, around))?;
    Some(match before_word(around.previous) {
        Before::Ordinal | Before::Nothing => reading.bare.unwrap_or(reading.lexeme),
        Before::Noun => reading.compound.unwrap_or(reading.lexeme),
        Before::Other => reading.lexeme,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::morphology::{Harmony::*, Inflection, Spoken, WordEnd::*};

    fn read(symbol: &str) -> Option<&'static str> {
        abbreviation(symbol).map(|lexeme| lexeme.output)
    }

    fn inflected(symbol: &str, inflection: Inflection) -> String {
        let lexeme = find(symbol).unwrap().lexeme;
        let mut spoken = Spoken::lexical(lexeme.output, lexeme.target);
        spoken.inflect(inflection);
        spoken.into_text()
    }

    fn around<'a>(previous: &'a str, next: &'a str) -> Around<'a> {
        Around {
            previous: Some(previous).filter(|word| !word.is_empty()),
            next: Some(next).filter(|word| !word.is_empty()),
        }
    }

    fn said_around(symbol: &str, previous: &str, next: &str) -> Option<&'static str> {
        abbreviation_around(symbol, around(previous, next)).map(|lexeme| lexeme.output)
    }

    #[test]
    fn every_abbreviation_is_written_once_and_said_in_letters() {
        let mut seen = HashSet::new();
        for entry in ABBREVIATIONS {
            assert!(seen.insert(entry.written), "{}", entry.written);
            assert!(
                entry
                    .said
                    .split(' ')
                    .all(|word| !word.is_empty() && word.chars().all(char::is_alphabetic)),
                "{}",
                entry.written
            );
        }
        // Every written form reads its own entry, and one that could be something else is read
        // in its own spelling only.
        for entry in ABBREVIATIONS {
            let reading = find(entry.written).unwrap();
            assert_eq!(reading.lexeme.output, entry.said, "{}", entry.written);
            assert!(
                entry.context == Context::Free || entry.capitals_key().is_none(),
                "{}",
                entry.written
            );
        }
        assert_eq!(
            table().lowercase.len(),
            ABBREVIATIONS
                .iter()
                .filter(|entry| lowercase_word(entry.written))
                .count()
        );
    }

    #[test]
    fn the_approved_entries_read_as_they_did() {
        for (symbol, expected) in [
            ("Dr.", Lexeme::same("doktor", BackRound, Voiced)),
            ("Prof.", Lexeme::same("profesör", FrontRound, Voiced)),
            (
                "vb.",
                Lexeme::distinct(
                    "ve benzeri",
                    Word::new("be", FrontFlat, Vowel),
                    Word::new("benzeri", FrontFlat, Vowel),
                ),
            ),
            (
                "TBMM",
                Lexeme::distinct(
                    "te be me me",
                    Word::new("me", FrontFlat, Vowel),
                    Word::new("me", FrontFlat, Vowel),
                ),
            ),
            (
                "PTT",
                Lexeme::distinct(
                    "pe te te",
                    Word::new("te", FrontFlat, Vowel),
                    Word::new("te", FrontFlat, Vowel),
                ),
            ),
            ("NATO", Lexeme::same("nato", BackRound, Vowel)),
            ("IBAN", Lexeme::same("iban", BackFlat, Voiced)),
            (
                "KDV",
                Lexeme::distinct(
                    "katma değer vergisi",
                    Word::new("ve", FrontFlat, Vowel),
                    Word::new("vergisi", FrontFlat, Possessive),
                ),
            ),
        ] {
            let lexeme = abbreviation(symbol).unwrap();
            assert_eq!(format!("{lexeme:?}"), format!("{expected:?}"), "{symbol}");
        }
    }

    #[test]
    fn capitals_read_a_dotted_abbreviation_unless_its_word_is_common() {
        for (symbol, said) in [
            ("DR.", "doktor"),
            ("PROF.", "profesör"),
            ("DOÇ.", "doçent"),
            ("CAD.", "caddesi"),
            ("SN.", "sayın"),
            ("MD.", "müdürlüğü"),
            ("ŞTİ.", "şirketi"),
            ("NO:", "numara"),
            ("TEL:", "telefon"),
            ("AR-GE", "araştırma geliştirme"),
        ] {
            assert_eq!(read(symbol), Some(said), "{symbol}");
        }
        // As written, case decides: `md.` is a law's article, `Md.` a directorate.
        assert_eq!(read("md."), Some("madde"));
        assert_eq!(read("Md."), Some("müdürlüğü"));
        // A common word in capitals, a lowercase spelling, a lone initial and one that could
        // be something else are not read without the words around them.
        for symbol in [
            "SOK.", "BUL.", "SAL.", "KAS.", "YAY.", "TEL.", "AV.", "sok.", "bul.", "dr.", "S.",
            "D.", "No", "abd", "Abd", "Bul.", "Av.", "s.", "c.", "min.", "AM", "mAh", "MAH", "vs.",
            "nolu", "Rus.",
        ] {
            assert_eq!(read(symbol), None, "{symbol}");
        }
    }

    #[test]
    fn one_that_could_be_something_else_is_read_only_where_nothing_else_fits() {
        for (previous, symbol, next, said) in [
            // A title before a name, a place after one, an abbreviation or a number.
            ("", "Av.", "Ayşe", "avukat"),
            ("Arş.", "Gör.", "Ece", "görevlisi"),
            ("Atatürk", "Bul.", "No:", "bulvarı"),
            ("Bilgisayar", "Müh.", "", "mühendis"),
            ("Müh.", "Böl.", "", "bölümü"),
            ("1234.", "Sok.", "No:", "sokak"),
            // A letter, a minimum and a number sign before a number.
            ("", "s.", "12'ye", "sayfa"),
            ("", "min.", "18", "minimum"),
            ("ve", "min.", "tutar", "minimum"),
            ("Kapı", "No", "5", "numarası"),
            ("", "NO", "7", "numara"),
            // A month or a time of day next to a number.
            ("5", "Kas.", "2024", "Kasım"),
            ("9", "AM", "", "ey em"),
            ("11", "ÖS", "", "öğleden sonra"),
            // A day or an example where the sentence goes on.
            ("", "Sal.", "günü", "salı"),
            ("(", "ör.", "belge", "örnek"),
            // A unit anywhere but before a number.
            ("", "MB", "cinsinden", "megabayt"),
        ] {
            assert_eq!(
                said_around(symbol, previous, next),
                Some(said),
                "{previous} {symbol} {next}"
            );
        }
        for (previous, symbol, next) in [
            // At the end of a sentence it is the word: find, see, release, muscle.
            ("bilmiyorum.", "Bul.", ""),
            ("Topla.", "Der.", ""),
            ("Der.", "Bul.", "Hemen."),
            ("", "Gör.", ""),
            ("onu.", "Sal.", ""),
            ("Kolunu", "Kas.", ""),
            ("Bu", "Av.", ""),
            // An answer, a list item and minutes.
            ("cevap", "c.", ""),
            ("", "d.", "Dördüncü"),
            ("30", "min.", "oldu"),
            ("I", "AM", "OK"),
            ("Çince", "GB", "18030"),
        ] {
            assert_eq!(
                said_around(symbol, previous, next),
                None,
                "{previous} {symbol} {next}"
            );
        }
    }

    #[test]
    fn a_number_abbreviation_ends_a_compound_after_a_noun() {
        for (word, before) in [
            ("Sipariş", Before::Noun),
            ("takip", Before::Noun),
            ("Caddesi", Before::Other),
            ("SOKAK", Before::Other),
            ("bir", Before::Other),
            ("ve", Before::Other),
            ("Cad.", Before::Other),
            ("5", Before::Other),
            ("12.", Before::Ordinal),
        ] {
            assert_eq!(before_word(Some(word)), before, "{word}");
        }
        assert_eq!(before_word(None), Before::Nothing);
        for symbol in ["No:", "No.", "no:", "no.", "Nr."] {
            assert_eq!(
                said_around(symbol, "Sipariş", ""),
                Some("numarası"),
                "{symbol}"
            );
            assert_eq!(
                said_around(symbol, "Caddesi", ""),
                Some("numara"),
                "{symbol}"
            );
        }
        // Without a compound or a bare form, the reading is the same wherever it stands; with
        // no word before it a place is said bare.
        assert_eq!(said_around("Dr.", "Sayın", ""), Some("doktor"));
        assert_eq!(said_around("Sok.", "12.", ""), Some("sokak"));
        assert_eq!(said_around("Sok.", "Gül", ""), Some("sokağı"));
        assert_eq!(said_around("Apt.", "", "veya"), Some("apartman"));
        assert_eq!(said_around("Apt.", "Yıldız", ""), Some("apartmanı"));
    }

    #[test]
    fn a_suffix_follows_the_last_word_said() {
        for (symbol, inflection, said) in [
            ("ABD", Inflection::Locative, "a be dede"),
            ("AB", Inflection::Genitive, "a benin"),
            ("THY", Inflection::Dative, "te he yeye"),
            ("KDV", Inflection::Dative, "katma değer vergisine"),
            ("Cad.", Inflection::Locative, "caddesinde"),
            ("T.C.", Inflection::Genitive, "Türkiye Cumhuriyetinin"),
            ("Bl.", Inflection::Accusative, "bloğu"),
            ("c.", Inflection::Accusative, "cildi"),
            ("Dr.", Inflection::Dative, "doktora"),
            ("NATO", Inflection::Ablative, "natodan"),
            ("GB", Inflection::Locative, "gigabaytta"),
        ] {
            assert_eq!(inflected(symbol, inflection), said, "{symbol}");
        }
        // A suffix written for the letters is checked against the letter name.
        let gigabyte = find("GB").unwrap().lexeme;
        assert_eq!(
            format!("{:?}", gigabyte.source),
            format!("{:?}", Word::new("be", FrontFlat, Vowel))
        );
    }
}
