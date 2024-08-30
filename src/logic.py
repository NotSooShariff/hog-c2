class CalculatorLogic:
    def __init__(self):
        pass
    
    def calculate(self, expression):
        try:
            result = str(eval(expression))
        except Exception as e:
            result = "Error"
        return result
