macro_rules! format_style {
    ($($Name:ident $name:literal $code:literal,)*) => {
        #[derive(Clone, Copy, Debug)]
        pub enum FormatStyle {
            $($Name,)*
        }

        impl FormatStyle {
            pub fn code(&self) -> &'static str {
                match self {
                    $(Self::$Name => $code,)*
                }
            }

            #[allow(dead_code)]
            pub fn from_code(s: &str) -> Option<Self> {
                match s {
                    $($code => Some(Self::$Name),)*
                    _ => None,
                }
            }

            pub fn from_name(s: &str) -> Option<Self> {
                let r = match s {
                    $($name => Some(Self::$Name),)*
                    _ => None,
                };

                r.or_else(|| Self::from_nick_name(s))
            }
        }
    };
}

impl FormatStyle {
    pub fn from_nick_name(s: &str) -> Option<Self> {
        let r = match s {
            "magenta" => Self::LightPurple,
            "dark_yellow" => Self::MinecoinGold,
            "quartz" => Self::MaterialQuartz,
            "iron" => Self::MaterialIron,
            "netherite" => Self::MaterialNetherite,
            "rand_char" => Self::Obfuscated,
            "redstone" => Self::MaterialRedstone,
            "copper" => Self::MaterialCopper,
            "dark_gold" => Self::MaterialGold,
            "emerald" => Self::MaterialEmerald,
            "diamond" => Self::MaterialDiamond,
            "lapis" => Self::MaterialLapis,
            "amethyst" => Self::MaterialAmethyst,
            _ => return None,
        };
        Some(r)
    }
}

format_style! {
    Black             "black"              "0",
    DarkBlue          "dark_blue"          "1",
    DarkGreen         "dark_green"         "2",
    DarkAqua          "dark_aqua"          "3",
    DarkRed           "dark_red"           "4",
    DarkPurple        "dark_purple"        "5",
    Gold              "gold"               "6",
    Gray              "gray"               "7",
    DarkGray          "dark_gray"          "8",
    Blue              "blue"               "9",
    Green             "green"              "a",
    Aqua              "aqua"               "b",
    Red               "red"                "c",
    LightPurple       "light_purple"       "d",
    Yellow            "yellow"             "e",
    White             "white"              "f",
    MinecoinGold      "minecoin_gold"      "g",
    MaterialQuartz    "material_quartz"    "h",
    MaterialIron      "material_iron"      "i",
    MaterialNetherite "material_netherite" "j",
    Obfuscated        "obfuscated"         "k",
    Bold              "bold"               "l",
    MaterialRedstone  "material_redstone"  "m",
    MaterialCopper    "material_copper"    "n",
    Italic            "italic"             "o",
    MaterialGold      "material_gold"      "p",
    MaterialEmerald   "material_emerald"   "q",
    Reset             "reset"              "r",
    MaterialDiamond   "material_diamond"   "s",
    MaterialLapis     "material_lapis"     "t",
    MaterialAmethyst  "material_amethyst"  "u",
}
