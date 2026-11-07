const path=require('path');
const dp=require('./node_modules/minecraft-data/minecraft-data/data/dataPaths.json').pc;
const base='./node_modules/minecraft-data/minecraft-data/data/';
const pick={766:'1.20.6',767:'1.21.1',768:'1.21.3',769:'1.21.4',770:'1.21.5',771:'1.21.6',772:'1.21.8',773:'1.21.9',774:'1.21.11',775:'26.1'};
const out={};
for(const [proto,ver] of Object.entries(pick)){
  const p=dp[ver]; if(!p||!p.protocol){console.error('no data',ver);continue}
  const j=require(base+p.protocol+'/protocol.json');
  out[proto]={version:ver};
  for(const state of ['handshaking','status','login','configuration','play']){
    const s=j[state]; if(!s) continue;
    out[proto][state]={};
    for(const dir of ['toClient','toServer']){
      const pk=s[dir].types.packet; // ["container",[{name:"name",type:["mapper",{type:"varint",mappings:{...}}]},...]]
      const m=pk[1][0].type[1].mappings; const o={};
      for(const [k,v] of Object.entries(m)) o[v]=parseInt(k,16);
      out[proto][state][dir]=o;
    }
  }
}
require('fs').writeFileSync('ids.json',JSON.stringify(out));
console.log('wrote',Object.keys(out).join(','));
