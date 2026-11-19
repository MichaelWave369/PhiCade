import { readFile } from "node:fs/promises";
const lock=JSON.parse(await readFile(new URL("../package-lock.json",import.meta.url),"utf8"));
const allowed=new Set(["MIT","Apache-2.0","Apache-2.0 OR MIT","MIT OR Apache-2.0","BSD-3-Clause","ISC","CC-BY-4.0"]);
const errors=[];let pinned=0;
for(const [path,p]of Object.entries(lock.packages)){
  if(!path.includes("node_modules")||p.link)continue;
  if(!p.version||!p.integrity)errors.push(`${path}: missing version or integrity`);
  if(!allowed.has(p.license))errors.push(`${path}: unreviewed license ${p.license??"missing"}`);
  pinned++;
}
if(errors.length){process.stderr.write(errors.join("\n")+"\n");process.exitCode=1;}else process.stdout.write(`PASS: ${pinned} resolved packages have pinned versions, integrity hashes and reviewed license expressions.\n`);
