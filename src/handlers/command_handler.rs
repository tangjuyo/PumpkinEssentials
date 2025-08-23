use async_trait::async_trait;
use pumpkin::{
    plugin::{EventHandler, EventPriority},
    server::Server,
    plugin::player::player_command_send::PlayerCommandSendEvent,
};
use std::sync::Arc;
use crate::managers::pm_manager::get_pm_manager;

pub struct CommandHandler;

#[async_trait]
impl EventHandler<PlayerCommandSendEvent> for CommandHandler {
    async fn handle_blocking(&self, server: &Arc<Server>, event: &mut PlayerCommandSendEvent) {
        let command = &event.command;
        
        // Vérifier si c'est une commande de message privé (/msg ou /tell)
        if command.starts_with("msg ") || command.starts_with("tell ") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() >= 3 {
                let target_name = parts[1];
                
                // Trouver le joueur cible
                if let Some(target_player) = server.get_player_by_name(target_name).await {
                    let sender_uuid = event.player.gameprofile.id;
                    let target_uuid = target_player.gameprofile.id;
                    
                    // Mettre à jour le dernier expéditeur pour le joueur cible
                    let pm_manager = get_pm_manager().await;
                    pm_manager.set_last_sender(target_uuid, sender_uuid).await;
                    log::info!("[Essentials] Updated last sender for {} to {}", target_name, event.player.gameprofile.name);
                }
            }
        }
    }
} 