#!/usr/bin/env python3
"""Train a Naive Bayes classifier for commit message classification.

Exports a simple JSON model that can be loaded by pmat for inference.
"""

import json
import re
import math
from collections import defaultdict
from pathlib import Path

def tokenize(text: str) -> list[str]:
    """Simple tokenization: lowercase, split on non-alphanumeric."""
    text = text.lower()
    # Remove common patterns that aren't useful
    text = re.sub(r'co-authored-by:.*', '', text)
    text = re.sub(r'[a-f0-9]{40}', '', text)  # commit hashes
    text = re.sub(r'refs?\s+\w+-\d+', '', text)  # ticket refs
    tokens = re.findall(r'[a-z]+', text)
    # Filter short tokens and stopwords
    stopwords = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'is', 'are', 'was', 'were', 'be', 'been', 'being', 'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'could', 'should', 'may', 'might', 'must', 'shall', 'can', 'need', 'dare', 'ought', 'used', 'this', 'that', 'these', 'those', 'it', 'its', 'from', 'by', 'as', 'not', 'all', 'each', 'every', 'both', 'few', 'more', 'most', 'other', 'some', 'such', 'no', 'nor', 'only', 'own', 'same', 'so', 'than', 'too', 'very'}
    return [t for t in tokens if len(t) > 2 and t not in stopwords]

def train_naive_bayes(data: list[dict]) -> dict:
    """Train a Multinomial Naive Bayes classifier."""
    # Build vocabulary and count frequencies
    vocab = defaultdict(int)
    class_counts = defaultdict(int)
    class_word_counts = defaultdict(lambda: defaultdict(int))

    for item in data:
        label = item['label']
        tokens = tokenize(item['message'])
        class_counts[label] += 1
        for token in tokens:
            vocab[token] += 1
            class_word_counts[label][token] += 1

    # Filter vocabulary to top N most frequent words
    top_n = 500
    sorted_vocab = sorted(vocab.items(), key=lambda x: -x[1])[:top_n]
    vocabulary = {word: idx for idx, (word, _) in enumerate(sorted_vocab)}

    # Calculate class priors and feature log probabilities
    total_docs = sum(class_counts.values())
    classes = sorted(class_counts.keys())
    class_priors = {c: math.log(class_counts[c] / total_docs) for c in classes}

    # Laplace smoothing for feature probabilities
    alpha = 1.0
    vocab_size = len(vocabulary)
    feature_log_probs = {}

    for c in classes:
        total_words = sum(class_word_counts[c].values())
        probs = []
        for word in vocabulary:
            count = class_word_counts[c].get(word, 0)
            prob = math.log((count + alpha) / (total_words + alpha * vocab_size))
            probs.append(prob)
        feature_log_probs[c] = probs

    return {
        'vocabulary': vocabulary,
        'classes': classes,
        'class_priors': class_priors,
        'feature_log_probs': feature_log_probs,
        'vocab_size': vocab_size,
        'alpha': alpha,
    }

def predict(model: dict, text: str) -> tuple[str, float]:
    """Predict class for a text."""
    tokens = tokenize(text)
    vocab = model['vocabulary']
    classes = model['classes']

    best_class = None
    best_score = float('-inf')
    scores = {}

    for c in classes:
        score = model['class_priors'][c]
        for token in tokens:
            if token in vocab:
                idx = vocab[token]
                score += model['feature_log_probs'][c][idx]
        scores[c] = score
        if score > best_score:
            best_score = score
            best_class = c

    # Convert log scores to probabilities for confidence
    max_score = max(scores.values())
    exp_scores = {c: math.exp(s - max_score) for c, s in scores.items()}
    total = sum(exp_scores.values())
    confidence = exp_scores[best_class] / total

    return best_class, confidence

def evaluate(model: dict, test_data: list[dict]) -> dict:
    """Evaluate model on test data."""
    correct = 0
    total = len(test_data)
    predictions = []

    for item in test_data:
        pred, conf = predict(model, item['message'])
        actual = item['label']
        correct += (pred == actual)
        predictions.append({
            'predicted': pred,
            'actual': actual,
            'confidence': conf,
            'correct': pred == actual
        })

    return {
        'accuracy': correct / total if total > 0 else 0,
        'correct': correct,
        'total': total,
        'predictions': predictions
    }

def main():
    # Load training data
    data_path = Path(__file__).parent / 'training-data.json'
    with open(data_path) as f:
        data = json.load(f)

    train_data = data.get('train', [])
    test_data = data.get('test', [])
    validation_data = data.get('validation', [])

    print(f"Training samples: {len(train_data)}")
    print(f"Validation samples: {len(validation_data)}")
    print(f"Test samples: {len(test_data)}")

    # Train model
    model = train_naive_bayes(train_data)
    print(f"\nVocabulary size: {model['vocab_size']}")
    print(f"Classes: {model['classes']}")

    # Evaluate
    if test_data:
        results = evaluate(model, test_data)
        print(f"\nTest accuracy: {results['accuracy']:.1%} ({results['correct']}/{results['total']})")

    if validation_data:
        val_results = evaluate(model, validation_data)
        print(f"Validation accuracy: {val_results['accuracy']:.1%}")

    # Export model
    export_model = {
        'version': '1.0',
        'type': 'naive_bayes',
        'vocabulary': model['vocabulary'],
        'classes': model['classes'],
        'class_priors': model['class_priors'],
        'feature_log_probs': model['feature_log_probs'],
        'metadata': {
            'train_samples': len(train_data),
            'vocab_size': model['vocab_size'],
            'alpha': model['alpha'],
        }
    }

    output_path = Path(__file__).parent / 'sovereign-stack-classifier.json'
    with open(output_path, 'w') as f:
        json.dump(export_model, f, indent=2)

    print(f"\nModel exported to: {output_path}")
    print(f"Model size: {output_path.stat().st_size / 1024:.1f} KB")

if __name__ == '__main__':
    main()
