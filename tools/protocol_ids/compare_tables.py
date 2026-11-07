#!/usr/bin/env python3
# usage: python3 compare_tables.py rust_packet_tables.json minecraft_data_ids.json
import json
import sys
rust=json.load(open(sys.argv[1])); mcd=json.load(open(sys.argv[2]))
CB={'add_entity':'spawn_entity','block_update':'block_change','chunk_batch_finished':'chunk_batch_finished','chunk_batch_start':'chunk_batch_start','container_close':'close_window','container_set_content':'window_items','container_set_data':'craft_progress_bar','container_set_slot':'set_slot','cookie_request':'cookie_request','disconnect':'kick_disconnect','disguised_chat':'profileless_chat','entity_position_sync':'sync_entity_position','forget_level_chunk':'unload_chunk','game_event':'game_state_change','keep_alive':'keep_alive','level_chunk_with_light':'map_chunk','login':'login','move_entity_pos':'rel_entity_move','move_entity_pos_rot':'entity_move_look','open_screen':'open_window','ping':'ping','player_abilities':'abilities','player_chat':'player_chat','player_combat_kill':'death_combat_event','player_info_update':'player_info','player_position':'position','remove_entities':'entity_destroy','remove_mob_effect':'remove_entity_effect','resource_pack_push':'add_resource_pack','respawn':'respawn','section_blocks_update':'multi_block_change','set_action_bar_text':'action_bar','set_cursor_item':'set_cursor_item','set_entity_data':'entity_metadata','set_entity_motion':'entity_velocity','set_equipment':'entity_equipment','set_experience':'experience','set_health':'update_health','set_held_slot':'held_item_slot','set_objective':'scoreboard_objective','set_player_inventory':'set_player_inventory','set_score':'scoreboard_score','set_subtitle_text':'set_title_subtitle','set_title_text':'set_title_text','start_configuration':'start_configuration','system_chat':'system_chat','teleport_entity':'entity_teleport','update_attributes':'entity_update_attributes','update_mob_effect':'entity_effect','block_changed_ack':'acknowledge_player_digging'}
SB={'accept_teleportation':'teleport_confirm','chat_command':'chat_command','chat':'chat_message','chunk_batch_received':'chunk_batch_received','client_command':'client_command','client_tick_end':'tick_end','configuration_acknowledged':'configuration_acknowledged','container_button_click':'enchant_item','container_click':'window_click','container_close':'close_window','cookie_response':'cookie_response','interact':'use_entity','keep_alive':'keep_alive','move_player_pos':'position','move_player_pos_rot':'position_look','move_player_rot':'look','move_player_status_only':'flying','place_recipe':'craft_recipe_request','player_abilities':'abilities','player_action':'block_dig','player_command':'entity_action','player_input':'player_input','player_loaded':'player_loaded','pong':'pong','rename_item':'name_item','resource_pack':'resource_pack_receive','select_trade':'select_trade','set_carried_item':'held_item_slot','sign_update':'update_sign','steer_vehicle':'steer_vehicle','swing':'arm_animation','use_item_on':'block_place','use_item':'use_item'}
tot=0
for p in ['766','767','768','769','770','771','772','773','774','775']:
    r=rust[p]; m=mcd[p]['play']; bad=[]; unknown=[]
    for k,name in CB.items():
        rv=r.get('cb_'+k)
        if name not in m['toClient']:
            unknown.append('cb_'+k+'('+name+' absent)'); continue
        if rv!=m['toClient'][name]: bad.append(('cb_'+k,rv,m['toClient'][name]))
    for k,name in SB.items():
        rv=r.get('sb_'+k)
        if name not in m['toServer']:
            if rv is not None and rv>=0: unknown.append('sb_'+k+'('+name+' absent in data, rust='+str(rv)+')')
            continue
        if rv!=m['toServer'][name]: bad.append(('sb_'+k,rv,m['toServer'][name]))
    tot+=len(bad)
    print(f"=== proto {p} ({mcd[p]['version']}): {len(bad)} mismatches, {len(unknown)} not-comparable")
    for b in bad[:60]: print("   ",b)
