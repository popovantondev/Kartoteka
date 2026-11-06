const fs=require('fs'),path=require('path');const root=path.resolve(__dirname,'..');
const repo='https://github.com/popovantondev/Kartoteka',site='https://popovantondev.github.io/Kartoteka/',catalogBase='https://popovantondev.github.io/popovantondev/';
const data=JSON.parse(fs.readFileSync(root+'/public-content.json','utf8'));
const languages=['ru','de','en'];
const esc=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
// German is the root language: index.html / README.md; the other languages carry their code in the file name.
const homeLabel={ru:'← На страницу программы',de:'← Zur Programmseite',en:'← Program page'};
const page=l=>l==='de'?'index.html':`index-${l}.html`,readme=l=>l==='de'?'README.md':`README.${l}.md`;
const setupUrl=`${repo}/releases/download/v0.5.0/Kartoteka-0.5.0-Setup.exe`,zipUrl=`${repo}/releases/download/v0.5.0/Kartoteka-0.5.0-Windows-x64.zip`,releaseUrl=`${repo}/releases/tag/v0.5.0`;
const kit=fs.readFileSync(root+'/docs/assets/kit/KIT_VERSION','utf8').trim();
// Query string of the demo script changes with the file, so browsers never keep an old copy.
const demoVersion=require('crypto').createHash('sha1').update(fs.readFileSync(root+'/docs/assets/demo.js','utf8').replace(/\r\n/g,'\n')).digest('hex').slice(0,8);
fs.mkdirSync(root+'/docs/assets',{recursive:true});
fs.mkdirSync(root+'/scripts',{recursive:true});
const sections=['install','language','first','custom','files','search','backup','keys','limits','build','changes','checks','rights'];
const stack='Rust · egui · SQLite';
for(const l of languages){const t=data[l];
 const links=languages.map(x=>`[${x.toUpperCase()}](${readme(x)})`).join(' · ');
 let md=`# ${t.title}\n\n${links}\n\n**${t.tagline} · 0.5.0 · Windows 11 x64**\n\n${t.intro}\n\n[${t.download}](${setupUrl}) · [${t.portable}](${zipUrl}) · [${t.guide}](${site}Guide-${l}.html) · [${t.catalog}](${catalogBase}${page(l)})\n\n![${t.title}](docs/assets/app-${l}.png)\n\n${t.screen}\n\n## ${t.features}\n\n${t.featureList.map(x=>'- '+x).join('\n')}\n`;
 for(const k of sections)md+=`\n## ${t[k]}\n\n${t[k+'Text']}\n`;
 md+=`\n[${t.feedback}](${repo}/issues/new) · [${t.source}](${repo})\n`;
 fs.writeFileSync(root+'/'+readme(l),md);
 fs.writeFileSync(root+`/docs/guide.${l}.md`,md.replaceAll('](docs/assets/','](assets/').replaceAll('(README.','(../README.'));
 fs.writeFileSync(root+`/docs/release-notes.${l}.md`,`# ${t.changes}\n\n${t.changesText}\n\n## ${t.install}\n\n${t.installText}\n\n## ${t.backup}\n\n${t.backupText}\n\n## ${t.limits}\n\n${t.limitsText}\n\n## SHA-256\n\n[SHA256SUMS.txt](${repo}/releases/download/v0.5.0/SHA256SUMS.txt)\n`);
 // Shared kit: head, header and footer are identical on landing page and guide.
 const head=(title,desc)=>`<!doctype html>\n<html lang="${l}">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width,initial-scale=1">\n<meta name="color-scheme" content="light dark">\n<meta name="description" content="${esc(desc)}">\n<title>${esc(title)}</title>\n<link rel="icon" href="assets/kartoteka.svg" type="image/svg+xml">\n<script src="assets/kit/kit-theme.js"></script><link rel="stylesheet" href="assets/kit/kit.css?v=${kit}"><script src="assets/kit/programs.js?v=${kit}" defer></script><script src="assets/kit/kit.js?v=${kit}" defer></script>\n<link rel="stylesheet" href="assets/site.css?v=${kit}">\n</head>\n`;
 const header=(self,file)=>`<body class="ppk-page ppk-prose">\n<a class="ppk-skip" href="#main">${esc(t.skip)}</a>\n<div class="ppk-nav-wrap" id="ppk-nav-wrap">\n  <nav class="ppk-nav" aria-label="${esc(t.nav)}">\n    <a class="ppk-brand" href="${page(l)}"><img src="assets/kartoteka.svg" alt="" width="34" height="34"><span>Kartoteka</span></a>\n    <span class="ppk-spacer"></span>\n    <a class="ppk-back" href="${catalogBase}${page(l)}">${esc(t.back)}</a>\n    <button class="ppk-search-btn" type="button" data-ppk-open-search aria-label="${esc(t.search)}"><span>${esc(t.searchText)}</span><kbd>${esc(t.kbd)}</kbd></button>\n    <div class="ppk-langs">${languages.map(x=>`<a href="${file(x)}" lang="${x}" hreflang="${x}"${x===l?' aria-current="page"':''}>${x.toUpperCase()}</a>`).join('')}</div>\n    <button class="ppk-icon-btn ppk-theme-btn" type="button" data-ppk-theme aria-label="${esc(t.theme)}"></button>\n    <a class="ppk-icon-btn ppk-gh-btn" href="${repo}" aria-label="GitHub"></a>\n  </nav>\n</div>\n`;
 const footer=`<footer class="ppk-footer">\n  <span>© 2026 Anton Popov · Kartoteka 0.5.0</span>\n  <nav aria-label="${esc(t.legal)}"><a href="${catalogBase}impressum.html">Impressum</a><a href="${catalogBase}datenschutz.html">Datenschutz</a><a href="${repo}">GitHub</a></nav>\n</footer>\n</body>\n</html>\n`;
 const alt=`${t.title} — ${t.screen}`;
 // Landing page.
 const term=t.term.map(([c,x])=>`<div class="ppk-${c}">${esc(x)}</div>`).join('');
 // Live demo skeleton; docs/assets/demo.js builds the controls (invented records, page memory only).
 const demoSection=`<section class="ppk-section ppk-demo-section ppk-reveal" id="demo" data-demo="kartoteka" aria-labelledby="demo-title">
<h2 id="demo-title">${esc(t.demoTitle)}</h2>
<p class="ppk-sub">${esc(t.demoSub)}</p>
<div class="ppk-demo"><div class="ppk-demo-head"><span>${esc(t.title)}</span><span class="ppk-demo-label">${esc(t.demoLabel)}</span></div>
<div class="ppk-demo-body"><div class="ppk-demo-main"><noscript><p class="ppk-demo-empty">${esc(t.demoNoJs)}</p></noscript></div></div></div>
</section>
`;
 const demoScript=`<script src="assets/demo.js?v=${demoVersion}" defer></script>\n`;
 const index=head(`${t.title} — ${t.tagline}`,t.intro)+header(l,page)+`<main class="ppk-shell" id="main">
<div class="ppk-hero">
<div>
<a class="ppk-pill" href="${releaseUrl}"><span class="ppk-new">${esc(t.new)}</span><span>${esc(t.pill)}</span> <span class="ppk-more">${esc(t.more)}</span></a>
<h1>${esc(t.title)}</h1>
<p class="ppk-lead">${esc(t.intro)}</p>
<span class="ppk-status">${esc(t.statusRelease)}</span> <span class="ppk-chip ppk-meta-chip">Windows 11 · x64</span>
<div class="ppk-actions">
<a class="ppk-btn ppk-btn-primary" href="${setupUrl}">${esc(t.download)}</a>
<a class="ppk-btn ppk-btn-gh" href="${repo}">${esc(t.source)} <span class="ppk-chip"><i class="ppk-dot"></i>${stack}</span></a>
<a class="ppk-btn" href="Guide-${l}.html">${esc(t.guide)} ↗</a>
</div>
<p class="ppk-requirements"><strong>${esc(t.reqTitle)}</strong>${esc(t.req)}</p>
</div>
<figure class="ppk-preview">
<div class="ppk-stage"><img src="assets/app-${l}.png" width="1360" height="860" alt="${esc(alt)}" decoding="async"></div>
<figcaption>${esc(t.screen)}</figcaption>
</figure>
</div>

<section class="ppk-bento-section ppk-section ppk-reveal">
<h2>${esc(t.bentoTitle)}</h2>
<p class="ppk-sub">${esc(t.bentoSub)}</p>
<div class="ppk-bento">
<div class="ppk-tile ppk-wide"><h3>${esc(t.install)}</h3><p>${esc(t.installText)}</p><div class="ppk-term" role="group" aria-label="${esc(t.install)}"><div class="ppk-term-bar"><i></i><i></i><i></i></div><div class="ppk-term-out">${term}</div></div></div>
<div class="ppk-tile"><h3>${esc(t.localTitle)}</h3><p>${esc(t.localText)}</p><span class="ppk-live">${esc(t.localLive)}</span></div>
<div class="ppk-tile"><h3>${esc(t.feats)}</h3><div class="ppk-chips">${t.featChips.map(x=>`<span>${esc(x)}</span>`).join('')}</div></div>
<div class="ppk-tile ppk-wide"><h3>${esc(t.checkTitle)}</h3><ul class="ppk-checks">${t.checkList.map(x=>`<li>${esc(x)}</li>`).join('')}</ul></div>
</div>
</section>

${demoSection}
<section class="ppk-section ppk-reveal">
<h2>${esc(t.features)}</h2>
<ul class="ppk-checks ppk-feature-list">${t.featureList.map(x=>`<li>${esc(x)}</li>`).join('')}</ul>
</section>

<section class="ppk-section ppk-reveal">
<div class="ppk-bento">
<div class="ppk-tile"><h3>${esc(t.limits)}</h3><p>${esc(t.limitsText)}</p></div>
<div class="ppk-tile"><h3>${esc(t.rights)}</h3><p>${esc(t.rightsText)}</p></div>
</div>
</section>
</main>
`+footer.replace('</body>',demoScript+'</body>');
 fs.writeFileSync(root+'/docs/'+page(l),index);
 // Guide page.
 const guide=head(`${t.title} — ${t.guide}`,t.intro)+header(l,x=>`Guide-${x}.html`)+`<main class="ppk-shell ppk-guide" id="main">
<p class="ppk-guide-home"><a class="ppk-btn" href="${page(l)}">${esc(homeLabel[l])}</a></p>
<h1>${esc(t.guide)}</h1>
<p class="ppk-lead">${esc(t.intro)}</p>
<div class="ppk-actions">
<a class="ppk-btn ppk-btn-primary" href="${setupUrl}">${esc(t.download)}</a>
<a class="ppk-btn" href="${zipUrl}">${esc(t.portable)}</a>
<a class="ppk-btn ppk-btn-gh" href="${repo}">${esc(t.source)} <span class="ppk-chip"><i class="ppk-dot"></i>${stack}</span></a>
</div>
<nav class="ppk-contents" aria-label="${esc(t.guide)}">${sections.map(k=>`<a href="#${k}">${esc(t[k])}</a>`).join('')}</nav>
${sections.map(k=>`<section id="${k}"><h2>${esc(t[k])}</h2><p>${esc(t[k+'Text'])}</p></section>`).join('\n')}
<figure class="ppk-guide-shot"><img src="assets/app-${l}.png" width="1360" height="860" alt="${esc(alt)}" decoding="async"><figcaption>${esc(t.screen)}</figcaption></figure>
</main>
`+footer;
 fs.writeFileSync(root+`/docs/Guide-${l}.html`,guide.replace(`assets/site.css?v=${kit}`,`assets/site.css?v=${kit}-g1`));
}
// Old German root name stays as a redirect, so existing links keep working.
fs.writeFileSync(root+'/docs/index-de.html',`<!doctype html>\n<html lang="de">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width,initial-scale=1">\n<script>location.replace('index.html'+location.hash)</script>\n<meta http-equiv="refresh" content="0; url=index.html">\n<link rel="canonical" href="${site}index.html">\n<meta name="robots" content="noindex">\n<link rel="icon" href="assets/kartoteka.svg" type="image/svg+xml">\n<title>Kartoteka — Weiterleitung</title>\n</head>\n<body>\n<p>Diese Seite ist umgezogen: <a href="index.html">Kartoteka — Startseite</a></p>\n<p><a href="index-en.html" lang="en" hreflang="en">English</a> · <a href="index-ru.html" lang="ru" hreflang="ru">Русский</a></p>\n</body>\n</html>\n`);
// Page-only styles: accent in all three theme paths plus a few rules for the guide; components come from assets/kit/kit.css.
const accent=(a,ink,tint)=>`--ppk-accent:${a};${ink?`--ppk-accent-ink:${ink};`:''}--ppk-tint:${tint};--ppk-hero:#14284f;--ppk-hero-end:#2b5cc0`;
fs.writeFileSync(root+'/docs/assets/site.css',`/* Page accent and page-only rules. Components and tokens come from assets/kit/kit.css. */
:root{${accent('#2458d6','','#eaf0ff')}}
:root[data-theme="dark"]{${accent('#86abff','#0b1a3a','#15264a')}}
@media (prefers-color-scheme:dark){:root:not([data-theme]){${accent('#86abff','#0b1a3a','#15264a')}}}
.ppk-meta-chip{margin-left:6px}
.ppk-feature-list{max-width:860px}
.ppk-guide{max-width:880px;padding-bottom:40px}
.ppk-guide-home{margin:24px 0 0}
.ppk-guide-home+h1{margin-top:20px}
.ppk-guide h1{margin:28px 0 12px;font-size:clamp(30px,5vw,44px);line-height:1.15}
.ppk-guide section{scroll-margin-top:84px;margin:30px 0}
.ppk-guide section>p{max-width:75ch}
.ppk-guide p,.ppk-tile p,.ppk-checks li,.ppk-lead{overflow-wrap:anywhere}
.ppk-contents{display:flex;flex-wrap:wrap;gap:8px;margin:26px 0 8px;padding:14px;border:1px solid var(--ppk-line);border-radius:var(--ppk-radius-m);background:var(--ppk-panel)}
.ppk-contents a{padding:4px 10px;border-radius:var(--ppk-radius-pill);background:var(--ppk-tint);font-size:14px;text-decoration:none}
.ppk-guide-shot{margin:36px 0 0}
.ppk-guide-shot img{display:block;width:100%;height:auto;border:1px solid var(--ppk-line);border-radius:var(--ppk-radius-m)}
.ppk-guide-shot figcaption{margin-top:10px;color:var(--ppk-muted);font-size:14px}
.ppk-stage img{display:block;width:100%;height:auto;border-radius:var(--ppk-radius-s)}
.demo-input{display:block;width:100%;margin-top:2px;padding:7px 9px;border:1px solid var(--ppk-line);border-radius:8px;background:var(--ppk-panel);color:var(--ppk-ink);font:inherit;font-size:14px}
.demo-input:focus-visible,.demo-open:focus-visible,.demo-card button:focus-visible{outline:2px solid var(--ppk-focus);outline-offset:2px}
textarea.demo-input{resize:vertical;min-height:56px}
.demo-count{margin:0 0 10px;color:var(--ppk-muted);font-size:14px}
.demo-open{display:block;width:100%;min-width:0;padding:2px 0;border:0;background:transparent;color:inherit;font:inherit;text-align:left;cursor:pointer;overflow-wrap:anywhere}
.demo-open small{color:var(--ppk-muted)}
.demo-card{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px 14px}
.demo-fld{display:block;min-width:0;font-size:13px;color:var(--ppk-muted)}
.demo-fld:nth-of-type(6),.demo-error,.demo-actions{grid-column:1/-1}
.demo-error{margin:0;min-height:1.2em;color:var(--ppk-warn-fg);font-size:14px;font-weight:650}
.demo-actions{display:flex;flex-wrap:wrap;gap:8px}
.demo-input[aria-invalid="true"]{border-color:var(--ppk-warn-fg)}
.ppk-demo-empty:empty{display:none}
@media (max-width:520px){.demo-card{grid-template-columns:1fr}}
`);
fs.writeFileSync(root+'/docs/.nojekyll','');
fs.writeFileSync(root+'/PUBLIC-DOCUMENTATION.md',`# Public documentation\n\nRU/DE/EN content is maintained in public-content.json. Run node scripts/generate-docs.cjs after edits and verify there is no output drift on a second run. README files, website pages and guides must agree with public-release.json and the actual release. Preserve user data, project identity and dependency licenses. Do not publish private databases, logs or personal data. Screenshots must come from the actual application and contain only demonstration data. A documentation check is not an application test.\n`);
fs.writeFileSync(root+'/public-content.json',JSON.stringify(data,null,2)+'\n');
