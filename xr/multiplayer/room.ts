import { XR_PROTOCOL, isPresence, type PeerMessage, type Presence } from "../protocol";
/** Host-owned room: membership and cabinet ownership never come from peer messages. */
export class ArcadeRoom {
  readonly players=new Map<string,{role:"controller"|"spectator";pose:Presence|null}>();
  owner:string|null=null;
  constructor(readonly id:string,readonly hostId:string){this.players.set(hostId,{role:"controller",pose:null});this.owner=hostId;}
  join(id:string){if(!/^[a-zA-Z0-9_-]{1,64}$/.test(id)||this.players.has(id)||this.players.size>=2)throw new Error("Room is full or identity is invalid");this.players.set(id,{role:"spectator",pose:null});}
  grant(operator:string,id:string){if(operator!==this.hostId||!this.players.has(id))throw new Error("Only the room host can assign the cabinet");this.owner=id;for(const [key,p]of this.players)p.role=key===id?"controller":"spectator";}
  leave(id:string){if(id===this.hostId){this.players.clear();this.owner=null;return;}this.players.delete(id);if(this.owner===id)this.grant(this.hostId,this.hostId);}
  update(id:string,pose:Presence){const p=this.players.get(id);if(!p||!isPresence(pose))throw new Error("Invalid room presence");p.pose=pose;}
  canInput(id:string){return this.owner===id&&this.players.get(id)?.role==="controller";}
  snapshot():PeerMessage{return{kind:"room",protocol:XR_PROTOCOL,roomId:this.id,owner:this.owner,players:[...this.players].map(([id,p])=>({id,...p}))};}
}
