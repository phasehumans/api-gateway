import nltk
from nltk.tokenize import word_tokenize
from nltk.corpus import stopwords
from nltk.stem import PorterStemmer, WordNetLemmatizer

# nltk.download('punkt')
# nltk.download('stopwords')
# nltk.download('wordnet')
# nltk.download('punkt_tab')

# use safe_download() > to avoid repeated download

text= "An LLM is an advanced AI trained on massive text datasets."
print(text)

tokens = word_tokenize(text.lower())

stop_words = set(stopwords.words('english'))

filter_tokens = []

for i in tokens:
    if i not in stop_words:
        filter_tokens.append(i)


stemmer = PorterStemmer()

stemmer_token = []
for i in filter_tokens:
    stemmer_token.append(stemmer.stem(i))


lemmatizer = WordNetLemmatizer()

lemmatize_token = []
for i in filter_tokens:
    lemmatize_token.append(lemmatizer.lemmatize(i))


print("Stemming >", stemmer_token)
print("Lemmatization > ", lemmatize_token)