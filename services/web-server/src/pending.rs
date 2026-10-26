use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tokio::sync::oneshot;

/// Requêtes de recommandation en attente de réponse, indexées par identifiant.
///
/// Un `Mutex` synchrone suffit : le verrou n'est jamais tenu à travers un
/// `await`, seulement le temps d'une insertion ou d'un retrait.
#[derive(Default)]
pub struct PendingRequests {
    next_id: AtomicU64,
    waiting: Mutex<HashMap<u64, oneshot::Sender<Vec<u32>>>>,
}

impl PendingRequests {
    /// Réserve un identifiant et le canal sur lequel la réponse arrivera.
    pub fn register(&self) -> (u64, oneshot::Receiver<Vec<u32>>) {
        let request_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = oneshot::channel();

        let mut waiting = self.waiting.lock().unwrap();
        // Purge les requêtes dont le client est parti avant la réponse : leur
        // handler a été abandonné sans passer par `forget`, rien d'autre ne les
        // retirerait.
        waiting.retain(|_, pending| !pending.is_closed());
        waiting.insert(request_id, sender);

        (request_id, receiver)
    }

    /// Transmet la réponse au handler en attente.
    ///
    /// Faux si l'identifiant est inconnu ou déjà résolu : le QoS AtLeastOnce
    /// autorise le broker à livrer deux fois la même réponse.
    pub fn resolve(&self, request_id: u64, movie_ids: Vec<u32>) -> bool {
        let sender = self.waiting.lock().unwrap().remove(&request_id);
        match sender {
            Some(sender) => sender.send(movie_ids).is_ok(),
            None => false,
        }
    }

    /// Retire une requête abandonnée : timeout écoulé ou publication échouée.
    pub fn forget(&self, request_id: u64) {
        self.waiting.lock().unwrap().remove(&request_id);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.waiting.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resout_la_requete_correspondante() {
        let pending = PendingRequests::default();
        let (request_id, receiver) = pending.register();

        assert!(pending.resolve(request_id, vec![550, 680]));
        assert_eq!(receiver.await.unwrap(), vec![550, 680]);
        assert_eq!(pending.len(), 0, "la requête résolue doit être retirée");
    }

    #[test]
    fn ignore_une_reponse_inconnue() {
        let pending = PendingRequests::default();
        let (request_id, _receiver) = pending.register();

        assert!(!pending.resolve(request_id + 42, vec![1]));
        assert_eq!(pending.len(), 1, "la requête en attente reste intacte");
    }

    #[test]
    fn ignore_un_doublon() {
        let pending = PendingRequests::default();
        let (request_id, _receiver) = pending.register();

        assert!(pending.resolve(request_id, vec![1]));
        assert!(
            !pending.resolve(request_id, vec![1]),
            "une seconde livraison du même message ne doit rien résoudre"
        );
    }

    #[test]
    fn oublie_apres_timeout() {
        let pending = PendingRequests::default();
        let (request_id, _receiver) = pending.register();

        pending.forget(request_id);

        assert_eq!(pending.len(), 0);
        assert!(!pending.resolve(request_id, vec![1]));
    }

    #[test]
    fn purge_les_clients_partis() {
        let pending = PendingRequests::default();

        // Client déconnecté : le receiver est abandonné sans `forget`.
        let (_abandoned, receiver) = pending.register();
        drop(receiver);

        let (_fresh, _receiver) = pending.register();

        assert_eq!(pending.len(), 1, "seule la requête vivante doit rester");
    }

    #[test]
    fn attribue_des_identifiants_distincts() {
        let pending = PendingRequests::default();
        let (first, _a) = pending.register();
        let (second, _b) = pending.register();

        assert_ne!(first, second);
    }
}
