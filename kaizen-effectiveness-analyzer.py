#!/usr/bin/env python3
"""
Kaizen Effectiveness Analyzer
Real-time analysis of optimization effectiveness with predictive modeling
"""

import json
import time
import numpy as np
from datetime import datetime, timedelta
from typing import List, Dict, Tuple, Optional
import matplotlib.pyplot as plt
import seaborn as sns
from collections import deque
import warnings
warnings.filterwarnings('ignore')

class KaizenEffectivenessAnalyzer:
    """Analyzes optimization effectiveness and predicts future improvements"""
    
    def __init__(self, window_size: int = 10):
        self.window_size = window_size
        self.improvement_history = deque(maxlen=window_size)
        self.complexity_reductions = deque(maxlen=window_size)
        self.time_per_iteration = deque(maxlen=window_size)
        self.success_rates = deque(maxlen=window_size)
        
    def load_metrics(self, metrics_file: str = "kaizen_metrics.json") -> Dict:
        """Load current metrics from file"""
        try:
            with open(metrics_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            return {"improvement_history": [], "optimizations_applied": []}
    
    def load_state(self, state_file: str = "optimization_state.json") -> Dict:
        """Load current optimization state"""
        try:
            with open(state_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            return {"iteration": 0, "total_improvement": 0.0}
    
    def analyze_trend(self, values: List[float]) -> Dict[str, float]:
        """Analyze trend in values using linear regression"""
        if len(values) < 2:
            return {"slope": 0.0, "r_squared": 0.0, "trend": "insufficient_data"}
        
        x = np.arange(len(values))
        y = np.array(values)
        
        # Linear regression
        coeffs = np.polyfit(x, y, 1)
        slope = coeffs[0]
        
        # R-squared
        y_pred = np.polyval(coeffs, x)
        ss_res = np.sum((y - y_pred) ** 2)
        ss_tot = np.sum((y - np.mean(y)) ** 2)
        r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0
        
        # Classify trend
        if slope > 0.1:
            trend = "improving"
        elif slope < -0.1:
            trend = "degrading"
        else:
            trend = "stable"
            
        return {
            "slope": float(slope),
            "r_squared": float(r_squared),
            "trend": trend
        }
    
    def predict_convergence(self, improvements: List[float], threshold: float = 0.05) -> Optional[int]:
        """Predict when improvements will fall below threshold"""
        if len(improvements) < 3:
            return None
            
        # Fit exponential decay model: y = a * exp(-b * x) + c
        x = np.arange(len(improvements))
        y = np.array(improvements)
        
        try:
            # Log transform for linear fitting
            log_y = np.log(y - np.min(y) + 0.001)
            coeffs = np.polyfit(x, log_y, 1)
            
            # Predict iterations until threshold
            if coeffs[0] < 0:  # Decaying
                iterations_to_threshold = int((np.log(threshold) - coeffs[1]) / coeffs[0])
                return max(0, iterations_to_threshold - len(improvements))
        except:
            pass
            
        return None
    
    def calculate_roi(self, state: Dict, metrics: Dict) -> Dict[str, float]:
        """Calculate return on investment metrics"""
        iteration = state.get("iteration", 1)
        total_improvement = state.get("total_improvement", 0.0)
        
        # Estimate time spent (assuming 5 minutes per iteration)
        time_spent_hours = (iteration * 5) / 60
        
        # Calculate metrics
        improvement_per_hour = total_improvement / max(time_spent_hours, 0.1)
        improvement_per_iteration = total_improvement / max(iteration, 1)
        
        # Efficiency score (0-100)
        expected_improvement = iteration * 5  # 5% per iteration expected
        efficiency = min(100, (total_improvement / max(expected_improvement, 1)) * 100)
        
        return {
            "total_improvement": total_improvement,
            "improvement_per_hour": improvement_per_hour,
            "improvement_per_iteration": improvement_per_iteration,
            "efficiency_score": efficiency,
            "time_invested_hours": time_spent_hours
        }
    
    def identify_patterns(self, metrics: Dict) -> List[Dict]:
        """Identify optimization patterns and their effectiveness"""
        patterns = {}
        
        # Analyze optimization history
        for opt in metrics.get("optimizations_applied", []):
            pattern = opt.get("pattern", "unknown")
            improvement = opt.get("improvement", 0.0)
            
            if pattern not in patterns:
                patterns[pattern] = {
                    "count": 0,
                    "total_improvement": 0.0,
                    "improvements": []
                }
            
            patterns[pattern]["count"] += 1
            patterns[pattern]["total_improvement"] += improvement
            patterns[pattern]["improvements"].append(improvement)
        
        # Calculate statistics for each pattern
        pattern_stats = []
        for pattern, data in patterns.items():
            if data["count"] > 0:
                avg_improvement = data["total_improvement"] / data["count"]
                std_improvement = np.std(data["improvements"]) if len(data["improvements"]) > 1 else 0
                
                pattern_stats.append({
                    "pattern": pattern,
                    "count": data["count"],
                    "avg_improvement": avg_improvement,
                    "std_improvement": std_improvement,
                    "total_impact": data["total_improvement"],
                    "reliability": 1.0 - (std_improvement / max(avg_improvement, 0.001))
                })
        
        # Sort by total impact
        return sorted(pattern_stats, key=lambda x: x["total_impact"], reverse=True)
    
    def generate_recommendations(self, state: Dict, metrics: Dict, trend: Dict) -> List[str]:
        """Generate actionable recommendations"""
        recommendations = []
        
        # Check trend
        if trend["trend"] == "degrading":
            recommendations.append("⚠️  Diminishing returns detected. Consider stopping after next iteration.")
        elif trend["trend"] == "stable":
            recommendations.append("📊 Improvements plateauing. Try adjusting optimization parameters.")
        
        # Check efficiency
        roi = self.calculate_roi(state, metrics)
        if roi["efficiency_score"] < 50:
            recommendations.append("🔧 Low efficiency detected. Review failed optimization attempts.")
        
        # Pattern analysis
        patterns = self.identify_patterns(metrics)
        if patterns:
            best_pattern = patterns[0]
            recommendations.append(
                f"✨ Most effective pattern: {best_pattern['pattern']} "
                f"(avg {best_pattern['avg_improvement']:.1f}% improvement)"
            )
        
        # Success rate
        success_rate = state.get("successful_optimizations", 0) / max(
            state.get("successful_optimizations", 0) + state.get("failed_attempts", 0), 1
        )
        if success_rate < 0.5:
            recommendations.append("🎯 Success rate below 50%. Consider more conservative optimization targets.")
        
        # Time-based recommendation
        current_hour = datetime.now().hour
        if 6 <= current_hour <= 8:
            recommendations.append("☀️  Morning hours - good time to review results and merge changes.")
        
        return recommendations
    
    def generate_report(self) -> str:
        """Generate comprehensive effectiveness report"""
        state = self.load_state()
        metrics = self.load_metrics()
        
        # Extract improvement history
        improvements = [h["improvement"] for h in metrics.get("improvement_history", [])]
        
        # Analyze trends
        trend = self.analyze_trend(improvements) if improvements else {"trend": "no_data"}
        
        # Calculate ROI
        roi = self.calculate_roi(state, metrics)
        
        # Get recommendations
        recommendations = self.generate_recommendations(state, metrics, trend)
        
        # Predict convergence
        iterations_to_convergence = self.predict_convergence(improvements) if improvements else None
        
        # Generate report
        report = f"""
╔══════════════════════════════════════════════════════════════════╗
║           KAIZEN EFFECTIVENESS ANALYSIS REPORT                   ║
║                    {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}                       ║
╚══════════════════════════════════════════════════════════════════╝

📊 PERFORMANCE METRICS
├─ Total Improvement: {roi['total_improvement']:.2f}%
├─ Improvement/Hour: {roi['improvement_per_hour']:.2f}%
├─ Improvement/Iteration: {roi['improvement_per_iteration']:.2f}%
├─ Efficiency Score: {roi['efficiency_score']:.1f}/100
└─ Time Invested: {roi['time_invested_hours']:.1f} hours

📈 TREND ANALYSIS
├─ Current Trend: {trend.get('trend', 'unknown').upper()}
├─ Trend Slope: {trend.get('slope', 0):.3f}
├─ R-squared: {trend.get('r_squared', 0):.3f}
└─ Convergence ETA: {f"{iterations_to_convergence} iterations" if iterations_to_convergence else "Unknown"}

🎯 OPTIMIZATION PATTERNS
"""
        
        patterns = self.identify_patterns(metrics)
        for i, pattern in enumerate(patterns[:5]):
            report += f"├─ {i+1}. {pattern['pattern']}: {pattern['avg_improvement']:.1f}% avg "
            report += f"({pattern['count']} applications)\n"
        
        report += "\n💡 RECOMMENDATIONS\n"
        for rec in recommendations:
            report += f"├─ {rec}\n"
        
        report += f"""
📊 RECENT IMPROVEMENTS
"""
        
        recent_improvements = improvements[-5:] if improvements else []
        for i, imp in enumerate(recent_improvements):
            bar = "█" * int(imp) + "░" * (20 - int(imp))
            report += f"├─ Iteration {len(improvements) - len(recent_improvements) + i + 1}: [{bar}] {imp:.1f}%\n"
        
        report += """
════════════════════════════════════════════════════════════════════
"""
        
        return report
    
    def plot_analysis(self, output_file: str = "kaizen_analysis.png"):
        """Generate visual analysis plots"""
        state = self.load_state()
        metrics = self.load_metrics()
        
        improvements = [h["improvement"] for h in metrics.get("improvement_history", [])]
        
        if not improvements:
            return
        
        fig, axes = plt.subplots(2, 2, figsize=(12, 8))
        fig.suptitle('Kaizen Optimization Analysis', fontsize=16)
        
        # Plot 1: Improvement over iterations
        ax1 = axes[0, 0]
        iterations = range(1, len(improvements) + 1)
        ax1.plot(iterations, improvements, 'b-', marker='o', markersize=6)
        ax1.set_xlabel('Iteration')
        ax1.set_ylabel('Improvement (%)')
        ax1.set_title('Improvement Trend')
        ax1.grid(True, alpha=0.3)
        
        # Add trend line
        if len(improvements) > 1:
            z = np.polyfit(iterations, improvements, 1)
            p = np.poly1d(z)
            ax1.plot(iterations, p(iterations), "r--", alpha=0.8, label=f'Trend: {z[0]:.3f}')
            ax1.legend()
        
        # Plot 2: Cumulative improvement
        ax2 = axes[0, 1]
        cumulative = np.cumsum(improvements)
        ax2.fill_between(iterations, 0, cumulative, alpha=0.3, color='green')
        ax2.plot(iterations, cumulative, 'g-', linewidth=2)
        ax2.set_xlabel('Iteration')
        ax2.set_ylabel('Cumulative Improvement (%)')
        ax2.set_title('Total Performance Gain')
        ax2.grid(True, alpha=0.3)
        
        # Plot 3: Pattern effectiveness
        ax3 = axes[1, 0]
        patterns = self.identify_patterns(metrics)
        if patterns:
            pattern_names = [p['pattern'][:15] for p in patterns[:5]]
            pattern_impacts = [p['total_impact'] for p in patterns[:5]]
            bars = ax3.bar(pattern_names, pattern_impacts, color='skyblue')
            ax3.set_xlabel('Optimization Pattern')
            ax3.set_ylabel('Total Impact (%)')
            ax3.set_title('Pattern Effectiveness')
            ax3.tick_params(axis='x', rotation=45)
            
            # Add value labels on bars
            for bar, value in zip(bars, pattern_impacts):
                height = bar.get_height()
                ax3.text(bar.get_x() + bar.get_width()/2., height,
                        f'{value:.1f}', ha='center', va='bottom')
        
        # Plot 4: Efficiency over time
        ax4 = axes[1, 1]
        if len(improvements) > 2:
            efficiency_scores = []
            for i in range(1, len(improvements) + 1):
                expected = i * 5  # 5% expected per iteration
                actual = sum(improvements[:i])
                efficiency = min(100, (actual / expected) * 100)
                efficiency_scores.append(efficiency)
            
            ax4.plot(iterations, efficiency_scores, 'purple', marker='s', markersize=5)
            ax4.axhline(y=100, color='gray', linestyle='--', alpha=0.5)
            ax4.axhline(y=50, color='red', linestyle='--', alpha=0.5)
            ax4.set_xlabel('Iteration')
            ax4.set_ylabel('Efficiency Score')
            ax4.set_title('Optimization Efficiency')
            ax4.set_ylim(0, 120)
            ax4.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()

def main():
    """Main analysis loop"""
    analyzer = KaizenEffectivenessAnalyzer()
    
    print("Starting Kaizen Effectiveness Analyzer...")
    print("Press Ctrl+C to stop\n")
    
    last_plot_time = time.time()
    
    try:
        while True:
            # Generate and display report
            report = analyzer.generate_report()
            print("\033[2J\033[H")  # Clear screen
            print(report)
            
            # Generate plots every 5 minutes
            if time.time() - last_plot_time > 300:
                analyzer.plot_analysis()
                last_plot_time = time.time()
                print("📊 Analysis plots saved to kaizen_analysis.png")
            
            # Wait before next update
            time.sleep(30)
            
    except KeyboardInterrupt:
        print("\n\nAnalysis stopped. Generating final report...")
        analyzer.plot_analysis()
        print("Final analysis saved to kaizen_analysis.png")

if __name__ == "__main__":
    main()