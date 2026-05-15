#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub struct SvgIcon {
    pub id: &'static str,
    pub label: &'static str,
    pub file: &'static str,
    pub category: IconCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconCategory {
    Browser,
    Navigation,
    Security,
    Brand,
    Creative3D,
}

pub const SVG_ICON_ROOT: &str = "SVG-Icon";

pub const SVG_ICONS: &[SvgIcon] = &[
    SvgIcon {
        id: "browser-close",
        label: "Cerrar",
        file: "browser-close.svg",
        category: IconCategory::Browser,
    },
    SvgIcon {
        id: "browser-minimize",
        label: "Minimizar",
        file: "browser-minimize.svg",
        category: IconCategory::Browser,
    },
    SvgIcon {
        id: "browser-maximize",
        label: "Maximizar",
        file: "browser-maximize.svg",
        category: IconCategory::Browser,
    },
    SvgIcon {
        id: "nav-back",
        label: "Atras",
        file: "nav-back.svg",
        category: IconCategory::Navigation,
    },
    SvgIcon {
        id: "nav-forward",
        label: "Adelante",
        file: "nav-forward.svg",
        category: IconCategory::Navigation,
    },
    SvgIcon {
        id: "nav-reload",
        label: "Recargar",
        file: "nav-reload.svg",
        category: IconCategory::Navigation,
    },
    SvgIcon {
        id: "nav-home",
        label: "Inicio",
        file: "nav-home.svg",
        category: IconCategory::Navigation,
    },
    SvgIcon {
        id: "security-lock",
        label: "Seguro",
        file: "security-lock.svg",
        category: IconCategory::Security,
    },
    SvgIcon {
        id: "security-shield",
        label: "Proteccion",
        file: "security-shield.svg",
        category: IconCategory::Security,
    },
    SvgIcon {
        id: "brand-youtube",
        label: "YouTube",
        file: "brand-youtube.svg",
        category: IconCategory::Brand,
    },
    SvgIcon {
        id: "brand-github",
        label: "GitHub",
        file: "brand-github.svg",
        category: IconCategory::Brand,
    },
    SvgIcon {
        id: "brand-x",
        label: "X",
        file: "brand-x.svg",
        category: IconCategory::Brand,
    },
    SvgIcon {
        id: "brand-openai",
        label: "OpenAI",
        file: "brand-openai.svg",
        category: IconCategory::Brand,
    },
    SvgIcon {
        id: "framework-threejs",
        label: "Three.js",
        file: "framework-threejs.svg",
        category: IconCategory::Creative3D,
    },
    SvgIcon {
        id: "framework-spline",
        label: "Spline",
        file: "framework-spline.svg",
        category: IconCategory::Creative3D,
    },
    SvgIcon {
        id: "framework-rive",
        label: "Rive",
        file: "framework-rive.svg",
        category: IconCategory::Creative3D,
    },
];

pub fn icon_by_id(id: &str) -> Option<&'static SvgIcon> {
    SVG_ICONS.iter().find(|icon| icon.id == id)
}
