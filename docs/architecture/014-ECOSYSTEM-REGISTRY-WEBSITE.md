# Catalyst — Ecosystem Registry & Website Architecture

**Document:** 014-ECOSYSTEM-REGISTRY-WEBSITE.md
**Status:** Architectural Direction
**Project:** Catalyst
**Related:** 005-PLUGIN-ARCHITECTURE.md, 013-VSCODE-PLUGIN-ARCHITECTURE-MAPPING.md

---

# 1. Purpose

Catalyst needs one official ecosystem catalog serving two clients:

1. the public Catalyst website;
2. the in-app Plugins & Themes browser.

The website and desktop application must not maintain separate catalogs.

```text
Plugin/Theme Authors
        ↓
Submission + Validation
        ↓
Official Catalyst Registry
       ↙             ↘
Public Website      Writer OS
Browse/Docs         Install/Update
```

# 2. Website Roles

The public Catalyst site should eventually provide:

- Writer OS download/release information;
- documentation;
- plugin developer documentation;
- theme developer documentation;
- Plugin API reference;
- theme token reference;
- searchable plugin catalog;
- searchable theme gallery;
- plugin/theme detail pages;
- version history/changelogs;
- compatibility information;
- permissions disclosure;
- source-code/homepage links;
- author/publisher information;
- install/open-in-Catalyst actions where supported;
- submission/publishing guidance;
- security/reporting information.

The marketing/download site and ecosystem catalog may share one domain and design system, but the registry remains a defined service/data contract rather than HTML scraped by the application.

# 3. Registry Is the Source of Catalog Truth

The official registry stores catalog metadata, not a user's project data.

The Writer OS queries a versioned registry endpoint or signed catalog artifact. It must never scrape website pages to discover plugins/themes.

The registry record should include at minimum:

- package ID;
- package type: plugin or theme;
- display name;
- description;
- author/publisher;
- current version;
- available versions;
- Plugin API/theme API compatibility;
- minimum Catalyst version;
- package artifact location;
- package integrity hash;
- source repository where provided;
- license identifier/text reference;
- permissions requested for plugins;
- executable/non-executable classification;
- screenshots/assets;
- release notes;
- publication/update timestamps;
- moderation/status information.

# 4. Plugins and Themes Stay Distinct

The catalog may present both under an Extensions/Ecosystem umbrella, but the UI must visibly distinguish:

**Plugin**
- executable capability;
- may request permissions;
- permission review required before activation.

**Theme**
- non-executable presentation package;
- no manuscript/network/process permissions;
- uses the public theme/token contract.

A theme package must not be allowed to masquerade as a plugin or acquire executable permissions.

# 5. Installation Flow

Target in-app flow:

```text
Browse
→ Open Details
→ Check compatibility
→ Review publisher/source/license
→ Review permissions (plugins)
→ Download package
→ Verify hash/signature policy
→ Validate manifest
→ Install
→ Explicitly grant requested permissions
→ Activate when required
```

Theme installation omits executable permission grants.

The browser website may support an `Open in Catalyst` deep link later, but the desktop host remains responsible for validation and installation.

# 6. Updates

Updates are package/version aware.

A plugin update that requests new permissions must not silently receive them. New permissions require explicit approval before the updated capability is enabled.

Theme updates cannot gain executable capabilities.

Compatibility checks occur before installation/update.

# 7. Package Hosting

Architecture must allow package artifacts to be hosted separately from website rendering.

This permits the public website implementation to change without breaking the desktop registry contract.

Initial hosting can be simple and static. The registry contract should not require a complex backend until submissions/accounts/moderation make one necessary.

# 8. Security and Trust

The registry improves discovery; it does not make community code inherently trusted.

The desktop host still enforces:

- manifest validation;
- API compatibility;
- package integrity;
- permission grants;
- project trust restrictions;
- Document Service boundaries;
- process/network/external-filesystem restrictions;
- safe mode and disablement.

Future signing/verified-publisher systems may add provenance without replacing these controls.

# 9. Community Submission

A future publishing workflow should validate:

- unique reverse-domain package ID;
- manifest schema;
- package paths;
- compatibility declarations;
- permissions;
- package integrity;
- license metadata;
- required README/description;
- prohibited executable behavior in themes;
- basic security/static checks where practical.

Human moderation may be added for first publication, reports, suspicious updates, or curated/featured listings.

# 10. Developer Experience

The website should house canonical developer documentation and starter material:

```text
Developers
├── Plugin Quick Start
├── VS Code → Catalyst Mapping
├── Plugin API
├── Manifest Reference
├── Permissions
├── Contribution Points
├── Sample Plugin
├── Theme Quick Start
├── Theme Manifest
├── Design Tokens
├── Theme Creator
├── Packaging
├── Testing
└── Publishing
```

# 11. Theme Gallery

Themes benefit from visual browsing. Theme listings should support screenshots/previews and declared light/dark compatibility.

The eventual Theme Creator should be able to export a package suitable for local installation and, after validation, community submission.

# 12. First-Party Packages

First-party optional plugins and themes should appear through the same package metadata model used by community packages.

Publisher status can be displayed, but first-party plugins still obey the public Plugin API and permission model.

# 13. Offline Behavior

Catalyst remains local-first.

Registry or website unavailability must not prevent:

- opening projects;
- writing;
- Story Development;
- using already installed compatible extensions;
- using installed themes.

Registry access is an optional network capability of the application, not a dependency for core writing.

# 14. Versioned Registry Contract

The registry schema/API is independently versioned.

The desktop application must reject unsupported registry/package formats safely rather than guessing.

A static JSON catalog is acceptable for the first implementation if it follows the versioned contract and can later migrate behind an API without changing package identity.

# 15. Domain / Deployment Direction

The ecosystem should have a dedicated official Catalyst web presence. Exact domain/subdomain and hosting are deployment decisions, not package identity.

Package IDs must therefore not depend on the current website hostname.

# 16. Release Sequencing

The registry website does not block the Core Preview.

Recommended sequence:

1. lock package/theme schemas;
2. ship local plugin/theme installation during development;
3. build canonical developer docs and sample packages;
4. define registry schema;
5. build public Catalyst website/download/docs;
6. add theme gallery and plugin catalog;
7. add in-app registry browser;
8. add community submission/moderation;
9. add signing/verified-publisher enhancements if warranted.

---

**Governing rule:** one ecosystem catalog, two clients — the website and Catalyst.
