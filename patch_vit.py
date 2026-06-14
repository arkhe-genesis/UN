import re

with open("cathedral_v14/cathedral_agi_production.py", "r") as f:
    content = f.read()

vit_class = """class ViTFeatureExtractor(nn.Module):
    \"\"\"Vision Transformer (ViT) pré-treinado para extração de características visuais.\"\"\"
    def __init__(self, output_dim=128):
        super().__init__()
        # Usa um ViT pré-treinado
        self.vit = models.vit_b_16(weights=models.ViT_B_16_Weights.DEFAULT)
        # Substitui a head final para mapear para output_dim
        self.vit.heads = nn.Linear(self.vit.heads.head.in_features, output_dim)

        # Transformações para ajustar observações não-visuais ou imagens brutas
        self.transform = models.ViT_B_16_Weights.DEFAULT.transforms()

    def forward(self, obs):
        if isinstance(obs, np.ndarray):
            obs = torch.from_numpy(obs).float()

        # Se for CartPole (vetor 1D de tamanho 4), convertemos num "patch" fictício RGB 224x224
        if obs.dim() == 1 or (obs.dim() == 2 and obs.shape[1] == 4):
            batch_size = 1 if obs.dim() == 1 else obs.size(0)
            # Cria uma imagem preta
            dummy_img = torch.zeros((batch_size, 3, 224, 224), dtype=torch.float)
            # Codifica a observação CartPole nos pixels do canal Red (canto superior esquerdo)
            if obs.dim() == 1:
                obs = obs.unsqueeze(0)
            dummy_img[:, 0, 0, :4] = obs
            obs = dummy_img

        # Se for imagem sem dimensão de batch
        if obs.dim() == 3:
            obs = obs.unsqueeze(0)

        # Se for canal no final (H, W, C), muda para (C, H, W)
        if obs.shape[-1] == 3 and obs.shape[1] != 3:
            obs = obs.permute(0, 3, 1, 2)

        # Redimensiona para 224x224 se necessário, simplificado aqui com interpolação se já for imagem real
        if obs.shape[-1] != 224 or obs.shape[-2] != 224:
            obs = F.interpolate(obs, size=(224, 224), mode='bilinear', align_corners=False)

        return self.vit(obs)
"""

content = re.sub(r'class FeatureExtractor\(nn\.Module\):.*?(?=\n# ============================================================================|\nclass CathedralAGI)', vit_class, content, flags=re.DOTALL)

content = content.replace('self.feature_extractor = FeatureExtractor(output_dim=128)', 'self.feature_extractor = ViTFeatureExtractor(output_dim=128)')

with open("cathedral_v14/cathedral_agi_production.py", "w") as f:
    f.write(content)
