# ACO-TSP: Otimização por Colônia de Formigas para o Problema do Caixeiro Viajante

[![Licença: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Uma implementação paralela e otimizada do algoritmo de Otimização por Colônia de Formigas (ACO) para resolver o clássico Problema do Caixeiro Viajante (TSP), com visualização interativa usando o motor de jogos Bevy.

## Características

- **Alto desempenho**: Implementação paralela usando múltiplos threads para processamento rápido
- **Visualização em tempo real**: Veja as formigas construindo e otimizando rotas
- **Interface intuitiva**: Controles simples para interação com o algoritmo
- **Configuração flexível**: Ajuste parâmetros do algoritmo através da interface ou arquivo de configuração
- **Temas claro/escuro**: Escolha o tema visual de sua preferência

## Tecnologias

- **[Rust](https://www.rust-lang.org/)**: Linguagem de programação segura e de alto desempenho
- **[Bevy](https://bevyengine.org/)**: Motor de jogos moderno baseado em ECS
- **[Bevy EGUI](https://github.com/mvlabat/bevy_egui)**: Integração do framework de UI Egui com Bevy
- **[Rayon](https://github.com/rayon-rs/rayon)**: Biblioteca para computação paralela em Rust
- **[Serde](https://serde.rs/)**: Framework para serialização/deserialização

## Instalação

Certifique-se de ter o Rust e Cargo instalados. Se ainda não tiver, instale via rustup:

```bash
# Instalar rustup (gerenciador de versões Rust)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Siga as instruções na tela para completar a instalação
# Em seguida, configure o ambiente para a sessão atual
source "$HOME/.cargo/env"
```

Em seguida, clone o repositório:

```bash
git clone https://github.com/seu-usuario/aco-tsp.git
cd aco-tsp
```

Execute o programa com:

```bash
cargo run --release
```

O flag `--release` é altamente recomendado para melhor performance.

## Como usar

1. Ajuste o número de pontos (cidades) usando o controle deslizante
2. Clique em "Gerar Pontos" para criar um novo problema TSP aleatório
3. (Opcional) Ajuste parâmetros avançados do algoritmo
4. Clique em "Iniciar" para começar a otimização
5. Observe em tempo real enquanto o algoritmo busca a melhor rota
6. A execução para automaticamente quando uma boa solução é encontrada

## Controles

- **Botão esquerdo do mouse + arrastar**: Mover a visualização
- **Roda do mouse**: Aumentar/diminuir zoom
- **Tecla T**: Alternar entre tema claro e escuro

## Configuração

O programa usa um arquivo de configuração `aco_config.json` que é criado automaticamente no primeiro uso. Você pode editar este arquivo para personalizar:

- Parâmetros do algoritmo (α, β, taxa de evaporação, etc.)
- Tamanho da janela e elementos visuais
- Limitação de FPS
- E mais configurações do algoritmo e visualização

## Solução de Problemas

Se você encontrar problemas de compilação ou execução, verifique:

1. **Versão do Rust**: Este projeto foi desenvolvido com Rust 1.73+ e Bevy 0.13.2
2. **Dependências**: Execute `cargo update` para atualizar as dependências
3. **Performance**: Use sempre `cargo run --release` para melhor desempenho
4. **Grandes conjuntos de dados**: Para problemas com muitos pontos (>100), o algoritmo otimiza automaticamente o uso de recursos, priorizando eficiência computacional. Isso pode resultar em iterações mais lentas, mas a convergência é mantida.

Para depurar problemas de paralelismo, considere executar com variáveis de ambiente para Rayon:

```bash
RAYON_NUM_THREADS=4 cargo run --release
```

## Como funciona

### Problema do Caixeiro Viajante

O TSP é um problema clássico de otimização combinatorial: dado um conjunto de cidades e as distâncias entre cada par, encontrar a rota mais curta que visita cada cidade exatamente uma vez e retorna à cidade de origem.

### Algoritmo de Otimização por Colônia de Formigas

ACO é um algoritmo meta-heurístico inspirado no comportamento de formigas. No contexto do TSP:

1. **Inicialização**: Formigas virtuais são colocadas aleatoriamente em cidades
2. **Construção de Caminhos**: Cada formiga constrói um caminho completo decidindo para qual cidade ir em seguida com base em:
   - Feromônio: preferência por caminhos com mais feromônio
   - Heurística: preferência por cidades mais próximas
3. **Atualização de Feromônio**: Após todas as formigas completarem seus caminhos:
   - O feromônio evapora em todas as arestas
   - Formigas depositam feromônio nas arestas usadas, proporcionalmente à qualidade de seus caminhos
4. **Repetição**: O processo é repetido até convergência ou limite de iterações

Esta implementação usa técnicas avançadas como paralelismo, estagnação adaptativa e estratégias elitistas para melhorar a performance e qualidade das soluções.

## Licença

Este projeto está licenciado sob a [Licença MIT](LICENSE) - veja o arquivo [LICENSE](LICENSE) para detalhes.
