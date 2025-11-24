.PHONY: setup run run-structured run-batch run-qwen demo clean

setup:
	python3 -m venv venv
	. venv/bin/activate && pip install --upgrade pip
	. venv/bin/activate && pip install -r requirements.txt

run:
	. venv/bin/activate && python simple_inference.py

run-qwen:
	. venv/bin/activate && python qwen_inference.py

run-structured:
	. venv/bin/activate && python structured_inference.py

run-batch:
	. venv/bin/activate && python batch_inference.py

demo:
	. venv/bin/activate && python presentation_demo.py

clean:
	rm -rf venv
	rm -rf __pycache__
	rm -rf *.json
	find . -type d -name "*.egg-info" -exec rm -rf {} +
