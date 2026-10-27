// Collect the unmodified notices shipped with the pinned dependency sources.
const fs=require('fs'),path=require('path'),os=require('os');
const root=path.resolve(__dirname,'..');
const cargo=process.env.CARGO_HOME||path.join(os.homedir(),'.cargo');
const registries=fs.readdirSync(path.join(cargo,'registry/src')).map(n=>path.join(cargo,'registry/src',n));
const lock=fs.readFileSync(path.join(root,'Cargo.lock'),'utf8');
const dest=path.join(root,'third-party');fs.mkdirSync(dest,{recursive:true});
let count=0,missing=[];
for(const block of lock.split('[[package]]').slice(1)){
 const name=/name = "([^"]+)"/.exec(block)?.[1],version=/version = "([^"]+)"/.exec(block)?.[1];
 if(!block.includes('registry+'))continue;
 const source=registries.map(r=>path.join(r,`${name}-${version}`)).find(p=>fs.existsSync(p));
 if(!source)continue; // Platform-only crates may not have been downloaded locally.
 const files=fs.readdirSync(source,{withFileTypes:true}).filter(e=>e.isFile()&&/^(licen[cs]e|copying|copyright|notice|unlicense)/i.test(e.name));
 if(!files.length)missing.push(`${name} ${version}`);
 for(const f of files){const to=path.join(dest,'crates',`${name}-${version}`,f.name);fs.mkdirSync(path.dirname(to),{recursive:true});fs.copyFileSync(path.join(source,f.name),to);count++;}
}
const llvmRoot=process.argv[2];if(llvmRoot){fs.copyFileSync(path.join(llvmRoot,'LICENSE.TXT'),path.join(dest,'LLVM-MinGW-LICENSE.TXT'));}
fs.writeFileSync(path.join(dest,'README.md'),`# Third-party notices\n\nUnmodified license and notice files from the dependencies pinned in Cargo.lock. This includes platform-specific packages and is not a claim that every listed package is linked into every binary. LLVM-MinGW-LICENSE.TXT covers the bundled LLVM runtime distribution. No separate license for the Kartoteka project code has been assigned.\n\nCollected files: ${count}. Packages whose top-level source directory did not contain a matching notice filename: ${missing.join(', ')||'none'}. Refer to the published crate source for its full terms.\n`);
console.log(`Collected ${count} dependency notice files; inspect ${missing.length} packages with no top-level notice.`);
