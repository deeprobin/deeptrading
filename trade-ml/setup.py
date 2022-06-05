from setuptools import setup, find_packages

setup(
    name='trademl',
    version='0.1.0',
    packages=find_packages(include=['trademl', 'trademl.*']),
    install_requires=[
        'numpy',
        'pandas',
        'keras',
        # 'tensorflow',
        'sklearn',
        'h5py',
        'gym==0.9.4',
        'chainer'
    ],
    extras_require={
        'dev': [
            'autopep8',
            'pycodestyle',
            'pytest',
            'pytest-cov',
            'pytest-pep8'
        ]
    }
)
