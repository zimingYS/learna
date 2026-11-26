from app.main import app


def test_app_importable() -> None:
    assert app.title == "learna-ai-service"
