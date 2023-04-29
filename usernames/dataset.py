from datasets import Dataset
import torch
from transformers import BertTokenizerFast


def gen():
    file = open("usernames.txt")
    line = file.readline()

    while line != "":
        yield {"name": line.strip("\n")}
        line = file.readline()


ds = Dataset.from_generator(gen)


tokenizer = BertTokenizerFast.from_pretrained("bert-base-cased")

ds.map(lambda x: (tokenizer(x["name"])))

ds.set_format(type="torch", columns=["name"])

dataloader = torch.utils.data.DataLoader(ds, batch_size=32)

print(next(iter(dataloader)))
