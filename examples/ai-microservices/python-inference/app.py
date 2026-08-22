def predict_sentiment(tokens: list[str]) -> dict:
    score = len(tokens) * 0.1
    return {"sentiment": "positive" if score > 0.2 else "neutral", "confidence": min(score, 1.0)}

if __name__ == "__main__":
    result = predict_sentiment(["fish", "build", "fast"])
    print(result)
