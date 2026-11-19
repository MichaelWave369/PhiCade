/* SPDX-License-Identifier: MIT
 * Original PhiCade no-content libretro homebrew. No ROM, BIOS or third-party code.
 * Minimal ABI declarations match https://github.com/libretro/libretro-common/include/libretro.h
 */
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#ifdef _WIN32
#define API __declspec(dllexport)
#else
#define API __attribute__((visibility("default")))
#endif
typedef bool (*environment_fn)(unsigned,void*);
typedef void (*video_fn)(const void*,unsigned,unsigned,size_t);
typedef size_t (*audio_batch_fn)(const int16_t*,size_t);
typedef void (*audio_fn)(int16_t,int16_t);
typedef void (*poll_fn)(void);
typedef int16_t (*input_fn)(unsigned,unsigned,unsigned,unsigned);
struct retro_system_info {const char *library_name,*library_version,*valid_extensions;bool need_fullpath,block_extract;};
struct retro_game_geometry {unsigned base_width,base_height,max_width,max_height;float aspect_ratio;};
struct retro_system_timing {double fps,sample_rate;};
struct retro_system_av_info {struct retro_game_geometry geometry;struct retro_system_timing timing;};
struct retro_game_info {const char*path;const void*data;size_t size;const char*meta;};
static environment_fn environment;
static video_fn video;
static audio_batch_fn audio_batch;
static audio_fn audio_sample;
static poll_fn poll_input;
static input_fn input;
static uint32_t pixels[320*240];
static int16_t sound[800*2];
static struct {unsigned hits,ammo;bool trigger;unsigned flash;} state;
API unsigned retro_api_version(void){return 1;}
API void retro_set_environment(environment_fn cb){bool support=true;environment=cb;environment(18,&support);}
API void retro_set_video_refresh(video_fn cb){video=cb;}
API void retro_set_audio_sample(audio_fn cb){audio_sample=cb;}
API void retro_set_audio_sample_batch(audio_batch_fn cb){audio_batch=cb;}
API void retro_set_input_poll(poll_fn cb){poll_input=cb;}
API void retro_set_input_state(input_fn cb){input=cb;}
API void retro_set_controller_port_device(unsigned port,unsigned device){(void)port;(void)device;}
API void retro_init(void){state.ammo=6;}
API void retro_deinit(void){}
API void retro_reset(void){memset(&state,0,sizeof(state));state.ammo=6;}
API void retro_get_system_info(struct retro_system_info *info){info->library_name="PhiCade Photon Range";info->library_version="1.0.0";info->valid_extensions="photon";info->need_fullpath=false;info->block_extract=false;}
API void retro_get_system_av_info(struct retro_system_av_info *av){av->geometry.base_width=av->geometry.max_width=320;av->geometry.base_height=av->geometry.max_height=240;av->geometry.aspect_ratio=4.0f/3.0f;av->timing.fps=60.0;av->timing.sample_rate=48000.0;}
API bool retro_load_game(const struct retro_game_info *info){unsigned format=1;(void)info;environment(10,&format);retro_reset();return true;}
API bool retro_load_game_special(unsigned type,const struct retro_game_info *info,size_t count){(void)type;(void)info;(void)count;return false;}
API void retro_unload_game(void){}
API unsigned retro_get_region(void){return 0;}
API void *retro_get_memory_data(unsigned id){(void)id;return NULL;}
API size_t retro_get_memory_size(unsigned id){(void)id;return 0;}
API size_t retro_serialize_size(void){return sizeof(state);}
API bool retro_serialize(void *data,size_t size){if(size!=sizeof(state))return false;memcpy(data,&state,size);return true;}
API bool retro_unserialize(const void *data,size_t size){if(size!=sizeof(state))return false;memcpy(&state,data,size);return true;}
API void retro_cheat_reset(void){}
API void retro_cheat_set(unsigned index,bool enabled,const char*code){(void)index;(void)enabled;(void)code;}
API void retro_run(void){
  poll_input();
  const int x=input(0,4,0,13),y=input(0,4,0,14);
  const bool off=input(0,4,0,15)!=0,trigger=input(0,4,0,2)!=0;
  if(input(0,4,0,16)||(off&&trigger&&!state.trigger))state.ammo=6;
  bool hit=false;
  if(trigger&&!state.trigger&&!off&&state.ammo){state.ammo--;if(x*x/4+y*y/4<2200*2200){state.hits++;state.flash=8;hit=true;}}
  state.trigger=trigger;
  const int px=(x+32767)*319/65534,py=(y+32767)*239/65534;
  for(int v=0;v<240;v++)for(int u=0;u<320;u++){
    const int d=(u-160)*(u-160)+(v-120)*(v-120);
    uint32_t color=(u%20==0||v%20==0)?0x15394b:0x081b27;
    if(d<25*25&&d>20*20)color=state.flash?0xf7b56c:0xc6f572;
    if(d<6*6)color=0xf7b56c;
    if(!off&&(u==px||v==py)&&((u-px)*(u-px)+(v-py)*(v-py)<100))color=0x70cbd8;
    if(v<5&&(unsigned)u<state.hits*10)color=0x00ff66; /* Observable score bar for ABI qualification. */
    if(v>226&&v<234&&u>=10&&(unsigned)u<10+state.ammo*25)color=0xc6f572;
    pixels[v*320+u]=color;
  }
  if(state.flash)state.flash--;
  for(int i=0;i<800;i++){const int16_t tone=hit?((i/24)%2?1800:-1800):0;sound[i*2]=sound[i*2+1]=tone;}
  video(pixels,320,240,320*sizeof(uint32_t));
  if(audio_batch)audio_batch(sound,800);else if(audio_sample)for(int i=0;i<800;i++)audio_sample(sound[i*2],sound[i*2+1]);
}
