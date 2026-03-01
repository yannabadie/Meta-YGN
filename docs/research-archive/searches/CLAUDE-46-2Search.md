# Architecture optimale pour un outil de métacognition IA universel

**MCP n'est pas le véhicule idéal en soi, mais il en constitue une brique essentielle.** L'architecture optimale pour un outil de métacognition à 7 couches est un **système hybride** : une bibliothèque SDK légère au cœur, MCP pour la découverte et la portabilité, le function calling natif pour l'exécution, et OpenTelemetry pour l'observabilité. Cette approche composite résout le problème central — le coût en tokens prohibitif de MCP en production — tout en préservant l'interopérabilité universelle. Les données montrent que MCP peut consommer **jusqu'à 82 000 tokens** (41 % d'une fenêtre de 200K) avant toute conversation, mais les techniques de chargement paresseux et d'encodage compact réduisent l'empreinte métacognitive à **25-80 tokens par appel**.

---

## Le vrai coût de MCP : un impôt silencieux sur chaque requête

Le protocole MCP injecte l'intégralité des schémas d'outils dans la fenêtre de contexte du LLM à chaque appel API. Un serveur MCP typique de 20 outils consomme environ **13 000-14 000 tokens** de définitions. Des mesures réelles dans Claude Code montrent un tableau accablant : les outils système intégrés (15 outils) coûtent 12 400 tokens, mais l'ajout de serveurs MCP externes fait exploser l'overhead à **82 000 tokens** — ne laissant que 5,8 % de la fenêtre de 200K pour le travail réel.

L'impact financier est concret. Un développeur avec seulement 4 serveurs MCP paie **~0,21 $ de surcoût** pour une question aussi simple que « 2+2 ? ». Une équipe DevOps de 5 personnes avec un usage standard génère **375 $/mois** en overhead pur de définitions d'outils. La latence augmente également de **0,24 ms par token d'entrée** — soit 2,4 secondes supplémentaires pour 10 000 tokens de définitions MCP.

Pour un outil de métacognition à 7 couches invoqué à haute fréquence, ce coût est rédhibitoire dans l'approche naïve. Sept outils avec des schémas complets représentent **700 à 3 850 tokens d'overhead par requête**, multipliés par chaque étape d'exécution de l'agent. La solution ne réside pas dans l'abandon de MCP, mais dans une architecture qui découple la découverte de l'exécution.

---

## MCPorter : pas un convertisseur, mais un précurseur du bon pattern

Le repo `steipete/mcporter` (~2 200 stars, MIT, TypeScript) est souvent décrit comme un « convertisseur MCP → function calling natif », mais cette caractérisation est inexacte. MCPorter est un **runtime TypeScript, CLI et générateur de code** qui implémente le pattern « code execution with MCP » — le même pattern préconisé par Anthropic et Cloudflare.

Son architecture repose sur cinq composants : un système de configuration à 4 niveaux (qui découvre automatiquement les serveurs MCP depuis Cursor, Claude Code, VS Code, etc.), un moteur runtime (`McpRuntime`) avec pool de connexions et cache, une couche transport unifiée (HTTP/SSE, stdio, daemon), un proxy serveur qui transforme les appels MCP en API TypeScript ergonomique, et un routeur CLI intelligent. La commande `emit-ts` génère des interfaces TypeScript à partir des schémas MCP ; `generate-cli` produit des CLI autonomes avec schémas embarqués.

**Les économies de tokens ne viennent pas de MCPorter directement**, mais du paradigme qu'il permet. Au lieu de charger chaque outil MCP comme une définition native dans le prompt, l'agent écrit du code TypeScript qui appelle l'API MCPorter. L'analyse d'Anthropic montre une réduction de **150 000 tokens à 2 000 tokens** (98,7 %) avec cette approche. Cloudflare va plus loin : ses 2 500+ endpoints API sont consolidés en **2 méta-outils consommant ~1 000 tokens** — une réduction de 99,9 %. La limitation principale de MCPorter reste l'exigence d'un environnement capable d'exécuter du code (Node.js 22+), ce qui n'est pas universel dans tous les contextes d'agents IA.

---

## Sept architectures comparées : laquelle sert le mieux la métacognition ?

L'analyse de sept approches alternatives révèle une hiérarchie claire pour un outil de métacognition à haute fréquence.

**L'approche SDK/bibliothèque** obtient le meilleur score global. En embarquant la métacognition comme une bibliothèque directement dans l'application (à la manière de Vercel AI SDK ou PydanticAI), on élimine tout overhead réseau, on bénéficie du typage statique, et on peut dynamiquement sélectionner quels outils inclure par requête. Le coût : un couplage avec le langage de programmation (TypeScript ou Python).

**L'approche hybride MCP + function calling natif** est le pattern émergent le plus prometteur. MCP assure la découverte des capacités métacognitives au runtime (via `tools/list`), puis les définitions sont « vendorisées » en outils natifs pour l'exécution. Vercel a créé `mcp-to-ai-sdk` exactement pour ce pattern. LangChain a reçu une feature request (#34130) pour un « MCP-to-Native Bridge » qui promet **>60 % de réduction de tokens**. En ne chargeant que 2-3 couches métacognitives pertinentes par étape (au lieu des 7), on passe de ~3 500 tokens à **~1 000-1 500 tokens** d'overhead.

**L'approche middleware/proxy** (type LiteLLM, 8 ms de latence au P95) excelle pour les déploiements enterprise nécessitant une abstraction provider centralisée. La métacognition peut être injectée comme augmentation de contexte plutôt que comme outils, éliminant totalement la « taxe tokens » des définitions. **L'approche sidecar/daemon** convient aux environnements Kubernetes avec besoin de persistance d'état métacognitif entre les requêtes.

Le function calling natif seul souffre du vendor lock-in (OpenAI utilise `parameters`, Anthropic `input_schema`, Google des types en majuscules) et de la « taxe contexte » identique à MCP. L'injection par prompt est la plus simple mais la moins fiable — aucune garantie de sortie structurée. Les plugins de frameworks agents (LangChain, CrewAI, AutoGen) offrent des écosystèmes riches mais imposent un lock-in framework souvent pire que le lock-in provider.

---

## Optimisation tokens : le vecteur métacognitif compact change la donne

Trois techniques combinées réduisent l'empreinte métacognitive de **97 %** par rapport à l'approche naïve.

**Le chargement paresseux à trois niveaux** (pattern Speakeasy « Dynamic Toolsets ») remplace le chargement de tous les schémas par trois méta-outils : `search_tools(query)` pour la découverte sémantique, `describe_tools(names)` pour le chargement à la demande, et `execute_tool(name, args)` pour l'exécution. Ce pattern atteint **96 % de réduction de tokens d'entrée** avec un taux de succès de 100 % sur des ensembles de 40 à 400 outils. Le pattern « Unlock » (Floris Fok, janvier 2026) offre une variante plus simple : un seul outil `unlock` par serveur MCP, activant les outils complets uniquement quand nécessaire — **45,5 % d'économie** même à petite échelle.

**L'encodage structuré du vecteur métacognitif** est la technique la plus puissante pour la métacognition spécifiquement. Au lieu de descriptions en langage naturel (100-200 tokens), un objet JSON compact encode l'état complet :

```json
{"c":0.85,"phase":"exec","step":"3/7","risk":"L","gaps":["auth"],"prev":"ok"}
```

Cet encodage capture confiance, phase d'exécution, progression, niveau de risque, lacunes identifiées et résultat de l'action précédente en **~30 tokens** — un gain de densité de **4-5x** par rapport au langage naturel. En mode ultra-compact (`META:c8u1p3d2r2`), on descend à **5-10 tokens**.

**Le prompt caching** exploite les mécanismes natifs d'OpenAI et d'Anthropic pour réduire de 80-90 % le coût des instructions métacognitives dans le system prompt après le premier appel. Combiné avec des descriptions d'outils minimisées (noms abrégés, schémas aplatis), le budget total par invocation métacognitive tombe à **75-415 tokens**, et à **0-15 tokens** quand la métacognition n'est pas activée (mode paresseux).

---

## Leçons des implémentations existantes de métacognition

L'analyse de Reflexion, Self-Refine, du think tool d'Anthropic et des serveurs MCP de thinking révèle un principe fondamental : **les outils métacognitifs les plus efficaces ne font rien**. Le think tool d'Anthropic — un simple schéma JSON qui crée un « scratchpad » pour la réflexion — améliore les performances de **54 %** sur τ-Bench (domaine aérien) sans aucune logique côté serveur. La clé réside dans le guidage par prompt : montrer au modèle *comment* réfléchir pour un domaine spécifique amplifie dramatiquement l'effet.

Reflexion (Shinn et al., NeurIPS 2023) apporte une dimension différente : la réflexion déclenchée par **feedback externe** (résultats de tests, signaux environnementaux) est plus fiable que l'auto-évaluation. Son architecture à trois composants (Acteur, Évaluateur, Modèle de Réflexion) avec mémoire épisodique montre que la **persistance des réflexions** entre les essais est cruciale. Self-Refine (Madaan et al.) démontre qu'un seul LLM peut générer, critiquer et affiner son propre travail avec ~20 % d'amélioration absolue — mais uniquement si le feedback est **actionnable et spécifique** plutôt que générique.

L'écosystème MCP a déjà produit plusieurs serveurs de thinking : `server-sequential-thinking` (référence officielle Anthropic), `thinking-patterns` (emmahyde, avec monitoring métacognitif et validation par schéma), et `mcp-server-mas-sequential-thinking` (approche multi-agents avec 6 perspectives cognitives). Le pattern Spring AI d'augmentation de schéma est particulièrement élégant : il ajoute `innerThought`, `confidence` et `memoryNotes` à chaque appel d'outil existant, capturant la métacognition *par outil* sans roundtrip supplémentaire.

---

## Le design AI-agnostic est atteignable mais exige de la discipline

Les formats de function calling des trois grands providers sont proches sans être identiques. OpenAI utilise `parameters` avec support du mode `strict`, Anthropic utilise `input_schema` avec des blocs `tool_use`/`tool_result`, Google utilise des types en majuscules (`STRING`, `OBJECT`) via OpenAPI 3.0 avec une limite de 128 déclarations par requête. Le noyau conceptuel (nom, description, paramètres typés en JSON Schema) est commun, mais les enveloppes divergent suffisamment pour nécessiter une couche d'adaptation.

Trois outils d'abstraction se distinguent. **LiteLLM** offre un appel `completion()` unifié pour 100+ providers avec 8 ms de latence au P95. **PydanticAI** fournit une interface Python type-safe avec `FallbackModel` pour le basculement automatique entre providers. **aisuite** (Andrew Ng) propose l'API la plus légère, modélisée sur le format OpenAI.

**OpenTelemetry est le standard de facto** pour l'observabilité AI-agnostic. Les conventions sémantiques GenAI (v1.37+) standardisent les attributs (`gen_ai.request.model`, compteurs de tokens, latence par span). OpenLLMetry et OpenLIT instrumentent automatiquement les appels LLM avec deux lignes de code. Pour l'outil de métacognition, OTEL permet de tracer chaque couche métacognitive comme un span, de suivre les coûts en tokens par couche, et d'alimenter la couche de calibration avec des données prédiction/réalité.

MCP et A2A (Google) sont complémentaires et non concurrents. MCP connecte les agents aux outils (communication verticale) ; A2A permet la communication entre agents (horizontale). Pour la couche 7 (métacognition collective), A2A est le protocole naturel pour coordonner les signaux métacognitifs entre agents, tandis que MCP distribue les capacités métacognitives individuelles.

---

## Recommandation architecturale : le système hybride à quatre couches

L'architecture optimale pour l'outil de métacognition ultime combine quatre couches complémentaires :

- **Couche 1 — SDK Core (bibliothèque embarquée)** : Une bibliothèque Python/TypeScript avec interface model-agnostic (style PydanticAI). Les 7 couches métacognitives sont implémentées comme des fonctions typées avec sortie structurée contrainte. Le vecteur métacognitif compact (~30 tokens) est l'unité de communication principale. Le chargement paresseux à trois niveaux garantit que seules les couches pertinentes sont actives par étape. Zéro overhead réseau.

- **Couche 2 — MCP pour la distribution et la découverte** : L'outil s'expose comme un serveur MCP pour la portabilité universelle (compatible Claude, ChatGPT, Gemini, Cursor, VS Code). Les définitions sont vendorisées en outils natifs pour l'exécution en production (pattern `mcp-to-ai-sdk`). Le serveur MCP sert de mécanisme de distribution et de découverte, pas d'exécution à chaud.

- **Couche 3 — OpenTelemetry pour l'observabilité** : Chaque invocation métacognitive génère un span OTEL avec attributs standardisés (confiance, phase, tokens consommés, latence). Les données alimentent la couche de calibration en boucle fermée. Compatible avec tout backend d'observabilité (Grafana, Datadog, Langfuse).

- **Couche 4 — A2A pour la métacognition collective** : La couche 7 (collective) utilise le protocole A2A pour partager les signaux métacognitifs entre agents. Les Agent Cards annoncent les capacités métacognitives ; les tâches A2A coordonnent la calibration inter-agents.

Le budget tokens par invocation avec cette architecture : **~50 tokens** pour le schéma d'outil minimal (en mode lazy, 0 quand non invoqué), **~30 tokens** pour le vecteur métacognitif structuré, et **~200-500 tokens** d'instructions system prompt (cachées à 90 % après le premier appel). Total effectif : **~80-130 tokens par invocation active** — contre 3 500+ tokens dans l'approche MCP naïve. Cela représente une **réduction de 96 %** tout en préservant l'universalité, la structure, et la persistance d'état nécessaires à une métacognition véritablement puissante.

## Conclusion

La question « MCP ou pas MCP ? » est mal posée. La réponse est « MCP et au-delà ». MCP résout brillamment le problème de distribution et d'interopérabilité — avec **97 millions de téléchargements mensuels** et une gouvernance sous la Linux Foundation, son écosystème est incontournable. Mais son modèle d'injection de tokens est inadapté à un outil appelé à haute fréquence. L'insight clé de cette recherche est que le think tool d'Anthropic — un outil qui ne fait littéralement rien — améliore les performances de 54 %, démontrant que la métacognition la plus puissante repose sur **l'espace structuré pour la réflexion**, pas sur la complexité de l'implémentation. L'architecture hybride recommandée exploite ce principe : un SDK léger au cœur, MCP pour la portabilité, le chargement paresseux pour l'efficience, et l'encodage compact pour la densité. Le vecteur métacognitif à 30 tokens est plus informatif que des paragraphes de réflexion en langage naturel — et 100 fois moins cher.