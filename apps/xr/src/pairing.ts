import { ManualPeer } from "../../../xr/bridge/peer";
import { ArcadeRoom } from "../../../xr/multiplayer/room";
import type { PeerMessage, Presence, XrAction } from "../../../xr/protocol";
import type { PhotonRange } from "../../../xr/lightgun/range";
import type { HollerCircuit } from "../../../xr/worldwalk/scene";
type Hooks={status:(s:string)=>void;choose:(i:number)=>void;setRemoteBridge:(v:HTMLVideoElement|null,input:(a:XrAction[])=>void,presence:(p:Presence)=>void,owns:boolean)=>void;showRemotePresence:(p:Presence|null)=>void;canvases:HTMLCanvasElement[];range:PhotonRange;world:HollerCircuit};
const el=<T extends HTMLElement>(id:string)=>document.getElementById(id) as T;
export function installPairing(h:Hooks){
  let peer:ManualPeer|null=null;let room:ArcadeRoom|null=null;let stream:MediaStream|null=null;let video:HTMLVideoElement|null=null;
  const cleanup=()=>{peer?.close();peer=null;stream?.getTracks().forEach(t=>t.stop());stream=null;if(video){video.pause();video.srcObject=null;video=null;}room=null;h.showRemotePresence(null);h.setRemoteBridge(null,()=>{},()=>{},false);};
  const setup=(p:ManualPeer)=>{
    p.onState=state=>{h.status(`Bridge: ${state}. ${state==="failed"||state==="disconnected"?"Create fresh pairing codes to reconnect.":""}`);if(["failed","disconnected","closed"].includes(state)){h.showRemotePresence(null);h.setRemoteBridge(video,()=>{},()=>{},false);}if(state==="paired"&&room)p.send(room.snapshot());};
    p.onLatency=ms=>{el("capability").textContent=`PAIRED · ${ms.toFixed(0)} MS RTT`;};
    p.onStream=s=>{if(!video){video=document.createElement("video");video.autoplay=true;video.playsInline=true;video.style.display="none";document.body.append(video);}video.srcObject=s;void video.play().catch(()=>h.status("Tap Connect again to enable stream audio."));h.setRemoteBridge(video,a=>p.input(a),pose=>p.presence(pose),p.role==="controller");h.choose(2);};
    p.onMessage=(m:PeerMessage)=>{
      if(m.kind==="status")h.status(m.text);
      if(m.kind==="presence"&&room){room.update("guest",m.pose);h.showRemotePresence(m.pose);p.send(room.snapshot());}
      if(m.kind==="room"){h.showRemotePresence(m.players.find(x=>x.id!=="guest")?.pose??null);h.setRemoteBridge(video,a=>p.input(a),pose=>p.presence(pose),m.owner==="guest");}
      if(m.kind==="input"&&room?.canInput("guest")){
        let u=-1,v=-1;for(const a of m.actions){if(a.kind==="axis"&&a.axis==="GUN_X")u=(a.value+32767)/65534;if(a.kind==="axis"&&a.axis==="GUN_Y")v=(a.value+32767)/65534;if(a.kind==="button"&&a.pressed){if(a.button==="GUN_TRIGGER")h.range.fire(u,v);if(a.button==="GUN_RELOAD")h.range.reload();const d:Record<string,[number,number]>={UP:[0,-1],DOWN:[0,1],LEFT:[-1,0],RIGHT:[1,0]};if(d[a.button])h.world.move(...d[a.button]);}}
      }
    };
  };
  el("connect").onclick=async()=>{try{cleanup();const p=peer=new ManualPeer();setup(p);h.status("Gathering local network candidates…");el<HTMLTextAreaElement>("answer").value=await p.answer(el<HTMLTextAreaElement>("offer").value);h.status("Answer ready. Return it to the PC, then accept it there.");}catch(e){h.status(String(e));cleanup();}};
  el("copy-answer").onclick=()=>{void navigator.clipboard.writeText(el<HTMLTextAreaElement>("answer").value).then(()=>h.status("Answer copied.")).catch(()=>h.status("Select and copy the answer manually."));};
  el("disconnect").onclick=()=>{cleanup();h.status("Disconnected. The host must approve a fresh session to reconnect.");};
  el("host-room").onclick=async()=>{try{cleanup();room=new ArcadeRoom(crypto.randomUUID(),"host");room.join("guest");room.grant("host","guest");stream=h.canvases[0].captureStream(30);const p=peer=new ManualPeer(stream);setup(p);el<HTMLTextAreaElement>("room-offer").value=await p.offer("controller",room.id);h.setRemoteBridge(null,()=>{},pose=>{room?.update("host",pose);if(room)p.send(room.snapshot());},false);h.status("Demo room ready. Guest controls Photon Range; this host spectates.");}catch(e){h.status(String(e));cleanup();}};
  el("accept-room").onclick=async()=>{try{if(!peer)throw new Error("Create a room offer first");await peer.accept(el<HTMLTextAreaElement>("room-answer").value);h.status("Guest accepted. One cabinet, one input owner.");}catch(e){h.status(String(e));}};
  window.addEventListener("pagehide",cleanup);
}
