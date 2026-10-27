const fs=require('fs'),path=require('path'),cp=require('child_process'),crypto=require('crypto');
const root=path.resolve(__dirname,'..');
const walk=d=>fs.readdirSync(d,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(path.join(d,e.name)):[path.join(d,e.name)]);
const digest=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex');
const files=walk(path.join(root,'docs')).concat(['README.md','README.ru.md','README.de.md','public-content.json','PUBLIC-DOCUMENTATION.md'].map(n=>path.join(root,n)));
const before=new Map(files.map(f=>[f,digest(f)]));
const generated=cp.spawnSync(process.execPath,[path.join(__dirname,'generate-docs.cjs')],{encoding:'utf8'});
if(generated.status!==0)throw Error(generated.stderr);
for(const [f,hash]of before)if(digest(f)!==hash)throw Error('Generated documentation drift: '+path.relative(root,f));
const version=/^version = "([^"]+)"/m.exec(fs.readFileSync(path.join(root,'Cargo.toml'),'utf8'))[1];
const metadata=JSON.parse(fs.readFileSync(path.join(root,'public-release.json'),'utf8'));
if(metadata.version!==version)throw Error('Public version mismatch');
for(const language of ['ru','de','en']){
 if(!metadata.languages.includes(language))throw Error('Missing language');
 for(const f of [language==='en'?'README.md':`README.${language}.md`,`docs/Guide-${language}.html`])if(!fs.readFileSync(path.join(root,f),'utf8').includes(version))throw Error('Missing version: '+f);
}
for(const file of files.filter(f=>f.endsWith('.html'))){
 const html=fs.readFileSync(file,'utf8');
 for(const m of html.matchAll(/(?:href|src)="([^"]+)"/g)){
  const link=m[1];if(/^(https?:|mailto:|#)/.test(link))continue;
  const target=path.resolve(path.dirname(file),link.split('#')[0]);if(!fs.existsSync(target))throw Error('Broken local link: '+link+' in '+file);
 }
 if(!/<html lang="(ru|de|en)"/.test(html))throw Error('Missing page language');
}
const catalog=JSON.parse(fs.readFileSync(path.join(root,'locales/ui.json'),'utf8'));
for(const [key,value]of Object.entries(catalog))for(const l of ['de','en'])if(!value[l]?.trim())throw Error('Missing translation: '+key);
console.log(`PASS: deterministic RU/DE/EN generation, local links, version ${version}, ${Object.keys(catalog).length} UI translations.`);
