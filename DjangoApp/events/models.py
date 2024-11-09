from django.db import models

class Event(models.Model):
    title = models.CharField(max_length=100)
    description = models.TextField()
    location = models.CharField(max_length=200)
    date = models.DateTimeField()
    total_tickets = models.PositiveIntegerField()

    def __str__(self):
        return self.title