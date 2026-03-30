"""配置管理模块"""

from pathlib import Path

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """统一配置"""

    # Ollama 配置
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5-coder:3b"

    # 数据目录
    data_root: Path = Path(__file__).parent.parent.parent / "data"

    # LLM 参数（可选）
    temperature: float = 0.1
    max_tokens: int = 1000

    class Config:
        env_prefix = ""
        extra = "ignore"
